use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};

use crate::device::operations;
use crate::device::{SmartData, StorageDevice};
use crate::platform;

pub fn print_banner() {
    println!(
        "\n{}",
        style("╔══════════════════════════════════════════════════════════╗").cyan()
    );
    println!(
        "{}",
        style("║     HDD Low Level Format Rust(ool)                       ║").cyan()
    );
    println!(
        "{}",
        style("║     Cross-platform Storage Device Utility                ║").cyan()
    );
    println!(
        "{}",
        style("╚══════════════════════════════════════════════════════════╝").cyan()
    );
    println!();
}

pub fn show_main_menu() -> Result<MainMenuChoice> {
    let choices = vec![
        "List Devices",
        "Low Level Format",
        "Quick Format",
        "Secure Erase (Multiple Passes)",
        "Verify Device",
        "View S.M.A.R.T. Data",
        "Exit",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select operation")
        .items(&choices)
        .default(0)
        .interact()?;

    Ok(match selection {
        0 => MainMenuChoice::ListDevices,
        1 => MainMenuChoice::LowLevelFormat,
        2 => MainMenuChoice::QuickFormat,
        3 => MainMenuChoice::SecureErase,
        4 => MainMenuChoice::VerifyDevice,
        5 => MainMenuChoice::ViewSmart,
        6 => MainMenuChoice::Exit,
        _ => MainMenuChoice::Exit,
    })
}

pub enum MainMenuChoice {
    ListDevices,
    LowLevelFormat,
    QuickFormat,
    SecureErase,
    VerifyDevice,
    ViewSmart,
    Exit,
}

pub fn list_devices(devices: &[StorageDevice]) {
    println!("\n{}", style("Available Storage Devices:").bold().cyan());
    println!("{}", style("─".repeat(80)).dim());

    if devices.is_empty() {
        println!("{}", style("No storage devices found.").yellow());
        return;
    }

    for (idx, device) in devices.iter().enumerate() {
        println!(
            "\n{} {}",
            style(format!("[{}]", idx + 1)).bold().green(),
            style(&device.model).bold()
        );
        println!("    Path:      {}", device.path);
        println!("    Serial:    {}", device.serial);
        println!("    Capacity:  {}", device.format_capacity());
        println!("    Type:      {}", device.device_type);
        println!("    Interface: {}", device.interface);
        println!(
            "    Removable: {}",
            if device.is_removable { "Yes" } else { "No" }
        );
    }

    println!();
}

pub fn select_device(devices: &[StorageDevice]) -> Result<usize> {
    if devices.is_empty() {
        anyhow::bail!("No devices available");
    }

    let items: Vec<String> = devices.iter().map(|d| d.display_name()).collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a device")
        .items(&items)
        .default(0)
        .interact()?;

    Ok(selection)
}

pub fn confirm_dangerous_operation(device: &StorageDevice, operation: &str) -> Result<bool> {
    println!("\n{}", style("⚠️  WARNING ⚠️").bold().red());
    println!("{}", style("─".repeat(80)).red());
    println!(
        "You are about to {} the following device:",
        style(operation).bold()
    );
    println!("  Device: {}", style(&device.model).yellow());
    println!("  Path:   {}", style(&device.path).yellow());
    println!("  Size:   {}", style(device.format_capacity()).yellow());
    println!();
    println!(
        "{}",
        style("This operation will PERMANENTLY ERASE ALL DATA!")
            .bold()
            .red()
    );
    println!("{}", style("This action CANNOT be undone!").bold().red());
    println!("{}", style("─".repeat(80)).red());
    println!();

    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Type YES to confirm you want to proceed")
        .default(false)
        .interact()?;

    if !confirmed {
        return Ok(false);
    }

    // Second confirmation
    println!();
    println!("{}", style("FINAL CONFIRMATION").bold().red());
    let final_confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Are you ABSOLUTELY SURE you want to {} {}?",
            operation, device.path
        ))
        .default(false)
        .interact()?;

    Ok(final_confirm)
}

pub fn perform_low_level_format(device: &StorageDevice) -> Result<()> {
    println!("\n{}", style("Starting low-level format...").cyan());

    let pb = ProgressBar::new(device.capacity);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .expect("Invalid progress bar template")
            .progress_chars("#>-"),
    );

    let pb_clone = pb.clone();
    operations::low_level_format(
        device,
        Some(Box::new(move |current, _total| {
            pb_clone.set_position(current);
        })),
    )?;

    pb.finish_with_message("Format completed successfully!");
    println!(
        "{}",
        style("✓ Device formatted successfully").green().bold()
    );

    Ok(())
}

pub fn perform_quick_format(device: &StorageDevice) -> Result<()> {
    println!("\n{}", style("Starting quick format...").cyan());

    operations::quick_format(device)?;

    println!(
        "{}",
        style("✓ Quick format completed successfully")
            .green()
            .bold()
    );

    Ok(())
}

pub fn perform_secure_erase(device: &StorageDevice, passes: u32) -> Result<()> {
    println!(
        "\n{}",
        style(format!("Starting secure erase ({} passes)...", passes)).cyan()
    );

    operations::secure_erase(device, passes)?;

    println!(
        "{}",
        style("✓ Secure erase completed successfully")
            .green()
            .bold()
    );

    Ok(())
}

pub fn perform_verify(device: &StorageDevice) -> Result<()> {
    println!("\n{}", style("Verifying device...").cyan());

    let pb = ProgressBar::new(device.capacity);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .expect("Invalid progress bar template")
            .progress_chars("#>-"),
    );

    let pb_clone = pb.clone();
    let result = operations::verify_device(
        device,
        Some(Box::new(move |current, _total| {
            pb_clone.set_position(current);
        })),
    )?;

    pb.finish_and_clear();

    if result {
        println!("{}", style("✓ Device verification passed").green().bold());
    } else {
        println!("{}", style("✗ Device verification failed").red().bold());
    }

    Ok(())
}

pub fn display_smart_data(device: &StorageDevice, smart_data: &SmartData) {
    println!(
        "\n{}",
        style(format!("S.M.A.R.T. Data for {}", device.model))
            .bold()
            .cyan()
    );
    println!("{}", style("─".repeat(80)).dim());

    println!(
        "Health Status: {}",
        match smart_data.health_status {
            crate::device::HealthStatus::Good => style("Good").green(),
            crate::device::HealthStatus::Warning => style("Warning").yellow(),
            crate::device::HealthStatus::Critical => style("Critical").red(),
            crate::device::HealthStatus::Unknown => style("Unknown").dim(),
        }
    );

    if let Some(temp) = smart_data.temperature {
        println!("Temperature:   {}°C", temp);
    }

    if let Some(hours) = smart_data.power_on_hours {
        println!("Power On Time: {} hours", hours);
    }

    if smart_data.attributes.is_empty() {
        println!("\n{}", style("No S.M.A.R.T. attributes available").yellow());
        println!(
            "{}",
            style("Note: S.M.A.R.T. data reading requires additional implementation").dim()
        );
    } else {
        println!("\nAttributes:");
        for attr in &smart_data.attributes {
            println!(
                "  {}: {} (Value: {}, Worst: {}, Threshold: {})",
                attr.name, attr.raw_value, attr.value, attr.worst, attr.threshold
            );
        }
    }

    println!();
}

pub fn check_prerequisites(device: &StorageDevice) -> Result<()> {
    // Check admin privileges
    if !platform::has_admin_privileges() {
        anyhow::bail!(
            "This operation requires administrator/root privileges.\n\
             Please run this program with elevated permissions:\n\
             - Linux/macOS: sudo ./hddllfrt\n\
             - Windows: Run as Administrator"
        );
    }

    // Check if device is mounted
    if platform::is_device_mounted(device)? {
        anyhow::bail!(
            "Device {} is currently mounted!\n\
             Please unmount all partitions before proceeding.",
            device.path
        );
    }

    Ok(())
}
