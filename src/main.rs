use anyhow::{Context, Result};
use console::style;
use std::process;

mod device;
mod platform;
mod ui;

use ui::MainMenuChoice;

fn main() {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    if let Err(e) = run() {
        eprintln!("{} {}", style("Error:").red().bold(), e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    ui::print_banner();

    // Check for admin privileges early
    if !platform::has_admin_privileges() {
        println!(
            "{}",
            style("⚠️  Warning: Not running with administrator/root privileges").yellow()
        );
        println!(
            "{}",
            style("Some operations will require elevated permissions.").yellow()
        );
        println!();
    }

    loop {
        let choice = ui::show_main_menu()?;

        match choice {
            MainMenuChoice::ListDevices => {
                handle_list_devices()?;
            }
            MainMenuChoice::LowLevelFormat => {
                handle_low_level_format()?;
            }
            MainMenuChoice::QuickFormat => {
                handle_quick_format()?;
            }
            MainMenuChoice::SecureErase => {
                handle_secure_erase()?;
            }
            MainMenuChoice::VerifyDevice => {
                handle_verify_device()?;
            }
            MainMenuChoice::ViewSmart => {
                handle_view_smart()?;
            }
            MainMenuChoice::Exit => {
                println!("\n{}", style("Thank you for using HDDLLFRT!").green());
                break;
            }
        }

        println!("\nPress Enter to continue...");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
    }

    Ok(())
}

fn handle_list_devices() -> Result<()> {
    println!("\n{}", style("Detecting devices...").cyan());

    let devices = platform::detect_devices().context("Failed to detect devices")?;

    ui::list_devices(&devices);

    Ok(())
}

fn handle_low_level_format() -> Result<()> {
    let devices = platform::detect_devices().context("Failed to detect devices")?;

    if devices.is_empty() {
        println!("{}", style("No devices found.").yellow());
        return Ok(());
    }

    ui::list_devices(&devices);

    let selection = ui::select_device(&devices)?;
    let device = &devices[selection];

    // Check prerequisites
    if let Err(e) = ui::check_prerequisites(device) {
        println!("{} {}", style("Error:").red().bold(), e);
        return Ok(());
    }

    // Get confirmation
    if !ui::confirm_dangerous_operation(device, "LOW LEVEL FORMAT")? {
        println!("{}", style("Operation cancelled.").yellow());
        return Ok(());
    }

    // Perform format
    ui::perform_low_level_format(device)?;

    Ok(())
}

fn handle_quick_format() -> Result<()> {
    let devices = platform::detect_devices().context("Failed to detect devices")?;

    if devices.is_empty() {
        println!("{}", style("No devices found.").yellow());
        return Ok(());
    }

    ui::list_devices(&devices);

    let selection = ui::select_device(&devices)?;
    let device = &devices[selection];

    // Check prerequisites
    if let Err(e) = ui::check_prerequisites(device) {
        println!("{} {}", style("Error:").red().bold(), e);
        return Ok(());
    }

    // Get confirmation
    if !ui::confirm_dangerous_operation(device, "QUICK FORMAT")? {
        println!("{}", style("Operation cancelled.").yellow());
        return Ok(());
    }

    // Perform format
    ui::perform_quick_format(device)?;

    Ok(())
}

fn handle_secure_erase() -> Result<()> {
    use dialoguer::{Input, theme::ColorfulTheme};

    let devices = platform::detect_devices().context("Failed to detect devices")?;

    if devices.is_empty() {
        println!("{}", style("No devices found.").yellow());
        return Ok(());
    }

    ui::list_devices(&devices);

    let selection = ui::select_device(&devices)?;
    let device = &devices[selection];

    // Check prerequisites
    if let Err(e) = ui::check_prerequisites(device) {
        println!("{} {}", style("Error:").red().bold(), e);
        return Ok(());
    }

    // Ask for number of passes
    let passes: u32 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Number of passes (1-10)")
        .default(3)
        .validate_with(|input: &u32| -> Result<(), &str> {
            if *input >= 1 && *input <= 10 {
                Ok(())
            } else {
                Err("Please enter a number between 1 and 10")
            }
        })
        .interact_text()?;

    // Get confirmation
    if !ui::confirm_dangerous_operation(device, &format!("SECURE ERASE ({} passes)", passes))? {
        println!("{}", style("Operation cancelled.").yellow());
        return Ok(());
    }

    // Perform secure erase
    ui::perform_secure_erase(device, passes)?;

    Ok(())
}

fn handle_verify_device() -> Result<()> {
    let devices = platform::detect_devices().context("Failed to detect devices")?;

    if devices.is_empty() {
        println!("{}", style("No devices found.").yellow());
        return Ok(());
    }

    ui::list_devices(&devices);

    let selection = ui::select_device(&devices)?;
    let device = &devices[selection];

    // Verify doesn't need write access, but still check admin
    if !platform::has_admin_privileges() {
        println!(
            "{}",
            style("⚠️  Warning: Running without elevated privileges may limit access").yellow()
        );
    }

    ui::perform_verify(device)?;

    Ok(())
}

fn handle_view_smart() -> Result<()> {
    let devices = platform::detect_devices().context("Failed to detect devices")?;

    if devices.is_empty() {
        println!("{}", style("No devices found.").yellow());
        return Ok(());
    }

    ui::list_devices(&devices);

    let selection = ui::select_device(&devices)?;
    let device = &devices[selection];

    println!("\n{}", style("Reading S.M.A.R.T. data...").cyan());

    match platform::read_smart_data(device) {
        Ok(smart_data) => {
            ui::display_smart_data(device, &smart_data);
        }
        Err(e) => {
            println!(
                "{} Failed to read S.M.A.R.T. data: {}",
                style("Error:").red().bold(),
                e
            );
        }
    }

    Ok(())
}
