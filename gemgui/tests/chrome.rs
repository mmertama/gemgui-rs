use std::env;

use which::which;

// collection of parameters that may speed up the chromium perf on headless
#[allow(unused)]
fn speed_params() -> Vec<&'static str> {
    let params = ["--disable-canvas-aa", // Disable antialiasing on 2d canvas
    "--disable-2d-canvas-clip-aa", // Disable antialiasing on 2d canvas clips
    "--disable-gl-drawing-for-tests", // BEST OPTION EVER! Disables GL drawing operations which produce pixel output. With this the GL output will not be correct but tests will run faster.
    "--disable-dev-shm-usage", // ???
    "--no-zygote", // wtf does that mean ?
    "--use-gl=swiftshader", // better cpu usage with --use-gl=desktop rather than --use-gl=swiftshader, still needs more testing.
    "--enable-webgl",
    "--hide-scrollbars",
    "--mute-audio",
    "--no-first-run",
    "--disable-infobars",
    "--disable-breakpad",
    "--window-size=1280,1024", // see defaultViewport
    "--no-sandbox", // meh but better resource comsuption
    "--disable-setuid-sandbox",
    "--ignore-certificate-errors",
    "--disable-extensions",
    "--disable-gpu",
    "--no-sandbox"];
    Vec::from(params)
}

#[allow(unused)]
pub fn headless_params(log: bool) -> Vec<String> {
    let binding = std::env::temp_dir();
    let dir = binding.to_str().unwrap();
    let mut params  = speed_params();
    let data_dir = String::from("--user-data-dir=") + dir;
    let mut p = vec!(
        "--headless", 
        "--remote-debugging-port=",
        "--user-data-dir=gemgui_test",
        "--disable-background-networking",
        "--disable-component-update",
        "--disable-update",
        "--disable-default-apps",
        "--disable-extensions",
        "--simulate-outdated-no-au='Tue, 31 Dec 2099 23:59:59 GMT'",
        "--no-first-run");
    p.push(data_dir.as_str());
    if cfg!(target_os = "windows") {
        p.push("--no-sandbox");
        p.push("--disable-gpu");
    }

    if log {
        p.push("--enable-logging --v=0");
    } else {
        p.push("--disable-logging");
    }
    
    params.append(&mut p);
    params.into_iter().map(|s| {String::from(s)} ).collect()
}

#[allow(unused)]
pub fn system_chrome() -> Option<(String, Vec<String>)> {
    if cfg!(target_os = "linux")  {
        for browser in vec!("chromium-chrome", "google-chrome").into_iter() {
            if which(&browser).is_ok() {
                return Some((String::from(browser), Vec::new()));
            }
        }
    }
    if cfg!(target_os = "macos") {
        // it has not been need for result, change if not found
        let result = which("google-chrome");
        return match(result) {
            Ok(path) => Some((path.to_str().unwrap().to_string(), Vec::new())),
            Err(_) => Some((String::from(r#"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"#), Vec::new()))
        }
    }
    if cfg!(target_os = "windows") {
         // Chrome installer wont update the PATH so we do an educated guess...
         let mut path = env::var("PATH").expect("PATH is not set!");
         if ! path.contains("chrome") {
             let path = format!("{};{}", r#"C:\Program Files (x86)\Google\Chrome\Application"#, path);
             env::set_var("PATH", path);
         }
        let result = which("chrome").expect("Tests requires that chrome is installed!");
        return Some(( result.to_str().unwrap().to_string(), Vec::new()));
    }
    None
}

fn kill_command(cmd: &str) -> String {
    return format!(r#"for pid in $(pgrep -f "{}"); do 
    for child in $(pgrep -P $pid -d " "); do 
        kill -TERM $child; 
    done; 
    kill -TERM $pid; 
    done"#, cmd)
    //return format!("for pid in $(pgrep -f '{}'); do pkill -TERM -P $pid; kill -TERM $pid; done", &cmd);
    //return vec!("-TERM".into(),
    //format!("-$(ps -o pgid= $(pgrep -f '{}') | grep -o '[0-9]*')", &cmd));
}

#[allow(unused)]
pub fn kill_headless() -> bool {
    let cmd: (&str, Vec<String>) = 
    if cfg!(target_os = "windows") {
        (("powershell.exe",
        vec!(r#"powershell.exe -command "Get-CimInstance -ClassName Win32_Process -Filter 'CommandLine LIKE ''%--headless%'' | %{Stop-Process -Id $_.ProcessId}""#.into())))
    } else if cfg!(target_os = "macos") {
        // old ("pkill", vec!(r#"-f \"(chrome)?(--headless)\""#))
        // ("pkill", vec!("-P", "-f", r#"Google Chrome.*headless"#))
        ("bash", vec!("-c".into(), kill_command(r#"Google Chrome.*headless"#)))
    } else {
        if(which("chromium-chrome").is_ok()) {
            ("bash", vec!("-c".into(), kill_command( r#"chromium.*headless"#)))
            //("pkill", vec!("-f", r#"chromium.*headless"#))
        } else {
            ("bash", vec!("-c".into(), kill_command(r#"chrome.*headless"#)))
            //("pkill", vec!("-f", r#"chrome.*headless"#))
        }
    };

    let output = std::process::Command::new(cmd.0)
    .args(&cmd.1)
    .output();


    match output {
        Ok(out) => {
            if !out.status.success() {
                if out.status.code().unwrap() != 1 {
                    println!("kill headless status: {}", out.status);
                    let cout = std::str::from_utf8(&out.stdout);
                    let cerr = std::str::from_utf8(&out.stderr);
                    println!("headless kill: out: {}\n err: {} call {} {}",
                    cout.unwrap_or("???"), cerr.unwrap_or("???"), cmd.0, cmd.1.join(" "));
                    return false;
                }
            } else {
                println!("UI killed ok");
            }
            true // here we get handle to spawned UI - not used now as exit is done nicely
        },
        Err(e) => {
            eprintln!("Kill error: {}", e);
            false
        }
    } 
}
