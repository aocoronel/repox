use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

static PRINT_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct Config {
    cmd: String,
    sub_dir: String,
    parallels: usize,
    repox_file: PathBuf,
    dev_dir: PathBuf,
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Config {
            cmd: self.cmd.clone(),
            sub_dir: self.sub_dir.clone(),
            parallels: self.parallels,
            repox_file: self.repox_file.clone(),
            dev_dir: self.dev_dir.clone(),
        }
    }
}

fn print_sync<F: FnOnce()>(f: F) {
    let lock = PRINT_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().unwrap();
    f();
}

fn read_repos(repox_file: &Path) -> io::Result<Vec<String>> {
    let file = File::open(repox_file)?;
    let reader = BufReader::new(file);
    Ok(reader
        .lines()
        .filter_map(Result::ok)
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect())
}

fn process_repo(config: &Config, repo: &str) {
    let repo_name = repo
        .trim_end_matches(".git")
        .rsplit('/')
        .next()
        .unwrap_or(repo);
    let local_repo = config.dev_dir.join(repo_name);

    let mut cmd = config.cmd.clone();

    if !local_repo.exists() && cmd != "clone" {
        cmd = "clone".to_string();
    } else if local_repo.exists() && cmd == "clone" {
        return;
    }

    let output = match cmd.as_str() {
        "status" => {
            let check = Command::new("git")
                .args(&["-C", local_repo.to_str().unwrap(), "status", "--porcelain"])
                .output();

            if let Ok(c) = check {
                if c.stdout.is_empty() {
                    return;
                }
            }

            Command::new("git")
                .args(&["-C", local_repo.to_str().unwrap(), "status"])
                .output()
        }

        "clone" => Command::new("git")
            .args(&["-C", config.dev_dir.to_str().unwrap(), "clone", repo])
            .output(),

        other => Command::new("git")
            .args(&["-C", local_repo.to_str().unwrap(), other])
            .output(),
    };

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            print_sync(|| {
                println!(
                    "\n\x1B[34m=== {}: {} ===\x1B[0m",
                    config.cmd.to_uppercase(),
                    repo
                );
                if !stdout.trim().is_empty() {
                    println!("{}", stdout.trim());
                }
                if !stderr.trim().is_empty() {
                    eprintln!("{}", stderr.trim());
                }
            });
        }
        Err(e) => {
            print_sync(|| {
                eprintln!(
                    "\n\x1B[31m[ERROR] {} failed on {}: {}\x1B[0m",
                    config.cmd, repo, e
                );
            });
        }
    }
}

fn run_in_parallel(config: Config, repos: Vec<String>) {
    let shared_repos = Arc::new(Mutex::new(repos));
    let mut handles = vec![];

    for _ in 0..config.parallels {
        let cfg = config.clone();
        let repos = Arc::clone(&shared_repos);

        let handle = thread::spawn(move || {
            loop {
                let repo = {
                    let mut locked = repos.lock().unwrap();
                    if locked.is_empty() {
                        break;
                    }
                    locked.pop()
                };

                if let Some(r) = repo {
                    process_repo(&cfg, &r);
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }
}

fn usage() {
    println!(
        "A git repository utility

Usage:
  repox FLAG <FLAG_INPUT> COMMAND SUB_DIRECTORY
  repox COMMAND github
  repox COMMAND codeberg

Commands:
  clone               Clone all repos
  fetch               Fetch all repos
  pull                Pull all repos
  status              Check status from all repos

Options:
  -h, --help           Displays this message and exits
  -p <PARALLEL>        Set parallels to use
  -c <FILE>            Use a specific repox file"
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut parallels = 5;
    let mut repox_file: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                usage();
                return;
            }
            "-p" => {
                i += 1;
                if i < args.len() {
                    parallels = args[i].parse().unwrap_or(5);
                }
            }
            "-c" => {
                i += 1;
                if i < args.len() {
                    repox_file = Some(PathBuf::from(&args[i]));
                }
            }
            _ => {
                break;
            }
        }
        i += 1;
    }

    if i + 1 > args.len() {
        eprintln!("ERROR: Missing command");
        exit(1);
    } else if i + 2 > args.len() {
        eprintln!("ERROR: Missing subdirectory");
        exit(1);
    }

    let cmd = args[i].clone();
    let sub_dir = args[i + 1].clone();

    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let repox_file = repox_file.unwrap_or_else(|| PathBuf::from(format!("{}/.repox", home)));

    if !repox_file.exists() {
        eprintln!("ERROR: No repox file found at {:?}", repox_file);
        exit(1);
    }

    let dev_dir = match env::var("DEV") {
        Ok(dev) => PathBuf::from(dev).join(&sub_dir),
        Err(_) => PathBuf::from(home).join("dev").join(&sub_dir),
    };

    if let Err(e) = fs::create_dir_all(&dev_dir) {
        eprintln!("ERROR: Failed to create dev directory: {}", e);
        exit(1);
    }

    let repos = match read_repos(&repox_file) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("ERROR: Could not read repox file: {}", e);
            exit(1);
        }
    };

    let config = Config {
        cmd,
        sub_dir,
        parallels,
        repox_file,
        dev_dir,
    };

    run_in_parallel(config, repos);
}
