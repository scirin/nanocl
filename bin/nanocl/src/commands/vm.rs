use std::thread;
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::time::Duration;

use ntex::rt;
use ntex::ws;
use ntex::time;
use ntex::util::Bytes;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use termios::{TCSANOW, tcsetattr, Termios, ICANON, ECHO};

use nanocl_utils::io_error::{IoResult, FromIo};
use nanocld_client::stubs::cargo::{OutputLog, OutputKind};

use crate::utils;
use crate::config::CliConfig;
use crate::models::{
  VmArg, VmCommand, VmCreateOpts, VmRow, VmRunOpts, VmPatchOpts, VmListOpts,
  VmInspectOpts,
};

use super::vm_image::exec_vm_image;

/// ## Exec vm create
///
/// Function executed when running `nanocl vm create`
/// It will create a new virtual machine but not start it
///
/// ## Arguments
///
/// * [cli_conf](CliConfig) The cli configuration
/// * [args](VmArg) The command arguments
/// * [options](VmCreateOpts) The command options
///
/// ## Return
///
/// * [Result](Result) The result of the operation
///   * [Ok](()) The operation was successful
///   * [Err](IoError) An error occured
///
pub async fn exec_vm_create(
  cli_conf: &CliConfig,
  args: &VmArg,
  options: &VmCreateOpts,
) -> IoResult<()> {
  let client = &cli_conf.client;
  let vm = options.clone().into();
  let vm = client.create_vm(&vm, args.namespace.clone()).await?;
  println!("{}", &vm.key);
  Ok(())
}

/// ## Exec vm ls
///
/// Function executed when running `nanocl vm ls`
/// It will list existing virtual machine and output them on stdout as a table.
///
/// ## Arguments
///
/// * [cli_conf](CliConfig) The cli configuration
/// * [args](VmArg) The command arguments
/// * [opts](VmListOpts) The command options
///
/// ## Return
///
/// * [Result](Result) The result of the operation
///   * [Ok](()) The operation was successful
///   * [Err](IoError) An error occured
///
pub async fn exec_vm_ls(
  cli_conf: &CliConfig,
  args: &VmArg,
  opts: &VmListOpts,
) -> IoResult<()> {
  let client = &cli_conf.client;
  let items = client.list_vm(args.namespace.clone()).await?;
  let rows = items.into_iter().map(VmRow::from).collect::<Vec<VmRow>>();
  match opts.quiet {
    true => {
      for row in rows {
        println!("{}", row.name);
      }
    }
    false => {
      utils::print::print_table(rows);
    }
  }
  Ok(())
}

/// ## Exec vm rm
///
/// Function executed when running `nanocl vm rm`
/// It will remove a virtual machine from the system
///
/// ## Arguments
///
/// * [cli_conf](CliConfig) The cli configuration
/// * [args](VmArg) The command arguments
/// * [names](Vec<String>) The list of virtual machine names to remove
///
/// ## Return
///
/// * [Result](Result) The result of the operation
///   * [Ok](()) The operation was successful
///   * [Err](IoError) An error occured
///
pub async fn exec_vm_rm(
  cli_conf: &CliConfig,
  args: &VmArg,
  names: &[String],
) -> IoResult<()> {
  let client = &cli_conf.client;
  for name in names {
    client.delete_vm(name, args.namespace.clone()).await?;
  }
  Ok(())
}

/// ## Exec vm inspect
///
/// Function executed when running `nanocl vm inspect`
/// It will inspect a virtual machine
/// and output the result on stdout as yaml, toml or json
/// depending on user configuration
///
/// ## Arguments
///
/// * [cli_conf](CliConfig) The cli configuration
/// * [args](VmArg) The command arguments
/// * [opts](VmInspectOpts) The command options
///
/// ## Return
///
/// * [Result](Result) The result of the operation
///   * [Ok](()) The operation was successful
///   * [Err](IoError) An error occured
///
pub async fn exec_vm_inspect(
  cli_conf: &CliConfig,
  args: &VmArg,
  opts: &VmInspectOpts,
) -> IoResult<()> {
  let client = &cli_conf.client;
  let vm = client
    .inspect_vm(&opts.name, args.namespace.clone())
    .await?;
  let display = opts
    .display
    .clone()
    .unwrap_or(cli_conf.user_config.display_format.clone());
  utils::print::display_format(&display, vm)?;
  Ok(())
}

/// ## Exec vm start
///
/// Function executed when running `nanocl vm start`
/// It will start a virtual machine that was previously created or stopped
///
/// ## Arguments
///
/// * [cli_conf](CliConfig) The cli configuration
/// * [args](VmArg) The command arguments
/// * [names](Vec<String>) The list of virtual machine names to start
///
/// ## Return
///
/// * [Result](Result) The result of the operation
///   * [Ok](()) The operation was successful
///   * [Err](IoError) An error occured
///
pub async fn exec_vm_start(
  cli_conf: &CliConfig,
  args: &VmArg,
  names: &[String],
) -> IoResult<()> {
  let client = &cli_conf.client;
  for name in names {
    if let Err(err) = client.start_vm(name, args.namespace.clone()).await {
      eprintln!("Failed to start vm {}: {}", name, err);
    }
  }
  Ok(())
}

/// ## Exec vm stop
///
/// Function executed when running `nanocl vm stop`
/// It will stop a virtual machine that was previously started
///
/// ## Arguments
///
/// * [cli_conf](CliConfig) The cli configuration
/// * [args](VmArg) The command arguments
/// * [names](Vec<String>) The list of virtual machine names to stop
///
/// ## Return
///
/// * [Result](Result) The result of the operation
///   * [Ok](()) The operation was successful
///   * [Err](IoError) An error occured
///
pub async fn exec_vm_stop(
  cli_conf: &CliConfig,
  args: &VmArg,
  names: &[String],
) -> IoResult<()> {
  let client = &cli_conf.client;
  for name in names {
    if let Err(err) = client.stop_vm(name, args.namespace.clone()).await {
      eprintln!("Failed to stop vm {}: {}", name, err);
    }
  }
  Ok(())
}

/// ## Exec vm run
///
/// Function executed when running `nanocl vm run`
/// It will create a new virtual machine, start it.
/// If the `attach` option is set, it will attach to the virtual machine console.
///
/// ## Arguments
///
/// * [cli_conf](CliConfig) The cli configuration
/// * [args](VmArg) The command arguments
/// * [options](VmRunOpts) The command options
///
/// ## Return
///
/// * [Result](Result) The result of the operation
///   * [Ok](()) The operation was successful
///   * [Err](IoError) An error occured
///
pub async fn exec_vm_run(
  cli_conf: &CliConfig,
  args: &VmArg,
  options: &VmRunOpts,
) -> IoResult<()> {
  let client = &cli_conf.client;
  let vm = options.clone().into();
  let vm = client.create_vm(&vm, args.namespace.clone()).await?;
  client.start_vm(&vm.name, args.namespace.clone()).await?;
  if options.attach {
    exec_vm_attach(cli_conf, args, &options.name).await?;
  }
  Ok(())
}

/// ## Exec vm patch
///
/// Function executed when running `nanocl vm patch`
/// It will patch a virtual machine with the provided options
///
/// ## Arguments
///
/// * [cli_conf](CliConfig) The cli configuration
/// * [args](VmArg) The command arguments
/// * [options](VmPatchOpts) The command options
///
/// ## Return
///
/// * [Result](Result) The result of the operation
///   * [Ok](()) The operation was successful
///   * [Err](IoError) An error occured
///
pub async fn exec_vm_patch(
  cli_conf: &CliConfig,
  args: &VmArg,
  options: &VmPatchOpts,
) -> IoResult<()> {
  let client = &cli_conf.client;
  let vm = options.clone().into();
  client
    .patch_vm(&options.name, &vm, args.namespace.clone())
    .await?;
  Ok(())
}

/// ## Exec vm attach
///
/// Function executed when running `nanocl vm attach`
/// It will attach to a virtual machine console
///
/// ## Arguments
///
/// * [cli_conf](CliConfig) The cli configuration
/// * [args](VmArg) The command arguments
/// * [name](&str) The name of the virtual machine to attach to
///
/// ## Return
///
/// * [Result](Result) The result of the operation
///   * [Ok](()) The operation was successful
///   * [Err](IoError) An error occured
///
pub async fn exec_vm_attach(
  cli_conf: &CliConfig,
  args: &VmArg,
  name: &str,
) -> IoResult<()> {
  let client = &cli_conf.client;
  /// How often heartbeat pings are sent
  const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
  let conn = client.attach_vm(name, args.namespace.clone()).await?;
  let (mut tx, mut rx) = mpsc::unbounded();
  // start heartbeat task
  let sink = conn.sink();
  rt::spawn(async move {
    loop {
      time::sleep(HEARTBEAT_INTERVAL).await;
      if sink.send(ws::Message::Ping(Bytes::new())).await.is_err() {
        return;
      }
    }
  });
  // // Get the current terminal settings
  let mut termios = Termios::from_fd(std::io::stdin().as_raw_fd())?;
  // Save a copy of the original terminal settings
  let original_termios = termios;
  // Disable canonical mode and echo
  termios.c_lflag &= !(ICANON | ECHO);
  // Redirect the output of the console to the TTY device
  let mut stderr = std::io::stderr();
  let mut stdout = std::io::stdout();
  // let mut tty_writer = std::io::BufWriter::new(tty_file);
  // std::io::copy(&mut stdout, &mut tty_writer)?;
  // Apply the new terminal settings
  tcsetattr(std::io::stdin().as_raw_fd(), TCSANOW, &termios)?;
  // start console read loop
  thread::spawn(move || loop {
    let mut input = [0; 1];
    if std::io::stdin().read(&mut input).is_err() {
      println!("Unable to read stdin");
      return;
    }
    let s = std::str::from_utf8(&input).unwrap();
    // send text to server
    if futures::executor::block_on(tx.send(ws::Message::Text(s.into())))
      .is_err()
    {
      return;
    }
  });
  // read console commands
  let sink = conn.sink();
  rt::spawn(async move {
    while let Some(msg) = rx.next().await {
      if sink.send(msg).await.is_err() {
        return;
      }
    }
  });
  // run ws dispatcher
  let sink = conn.sink();
  let mut rx = conn.seal().receiver();
  while let Some(frame) = rx.next().await {
    match frame {
      Ok(ws::Frame::Binary(text)) => {
        let output =
          serde_json::from_slice::<OutputLog>(&text).map_err(|err| {
            err.map_err_context(|| "Unable to serialize output")
          })?;
        match &output.kind {
          OutputKind::StdOut => {
            stdout.write_all(output.data.as_bytes())?;
            stdout.flush()?;
          }
          OutputKind::StdErr => {
            stderr.write_all(output.data.as_bytes())?;
            stdout.flush()?;
          }
          OutputKind::Console => {
            stdout.write_all(output.data.as_bytes())?;
            stdout.flush()?;
          }
          _ => {}
        }
      }
      Ok(ws::Frame::Ping(msg)) => {
        sink
          .send(ws::Message::Pong(msg))
          .await
          .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
      }
      Err(_) => break,
      _ => (),
    }
  }
  // Restore the original terminal settings
  tcsetattr(std::io::stdin().as_raw_fd(), TCSANOW, &original_termios)?;
  Ok(())
}

/// ## Exec vm
///
/// Function executed when running `nanocl vm`
/// It will execute the subcommand passed as argument
///
/// ## Arguments
///
/// * [cli_conf](CliConfig) The cli configuration
/// * [args](VmArg) The command arguments
///
/// ## Return
///
/// * [Result](Result) The result of the operation
///   * [Ok](()) The operation was successful
///   * [Err](IoError) An error occured
///
pub async fn exec_vm(cli_conf: &CliConfig, args: &VmArg) -> IoResult<()> {
  let client = &cli_conf.client;
  match &args.command {
    VmCommand::Image(args) => exec_vm_image(client, args).await,
    VmCommand::Create(options) => exec_vm_create(cli_conf, args, options).await,
    VmCommand::List(opts) => exec_vm_ls(cli_conf, args, opts).await,
    VmCommand::Remove(opts) => exec_vm_rm(cli_conf, args, &opts.names).await,
    VmCommand::Inspect(opts) => exec_vm_inspect(cli_conf, args, opts).await,
    VmCommand::Start(opts) => exec_vm_start(cli_conf, args, &opts.names).await,
    VmCommand::Stop(opts) => exec_vm_stop(cli_conf, args, &opts.names).await,
    VmCommand::Run(options) => exec_vm_run(cli_conf, args, options).await,
    VmCommand::Patch(options) => exec_vm_patch(cli_conf, args, options).await,
    VmCommand::Attach { name } => exec_vm_attach(cli_conf, args, name).await,
  }
}
