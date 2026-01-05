use train::{LedController, AppState, create_router, GREEN_LEDS, AMBER_LEDS, RED_LEDS};
use clap::{Parser, Subcommand};
use tokio::net::TcpListener;

#[derive(Parser)]
#[command(name = "train")]
#[command(about = "Train Set Control System", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run tests on hardware components
    Test {
        #[command(subcommand)]
        component: TestComponent,
    },
    /// Start the web server
    Server {
        /// Port to listen on (default: 8080)
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
        /// Host to bind to (default: 0.0.0.0)
        #[arg(short = 'H', long, default_value = "0.0.0.0")]
        host: String,
    },
}

#[derive(Subcommand)]
enum TestComponent {
    /// Test LED indicators
    Led {
        #[command(subcommand)]
        test: LedTest,
    },
}

#[derive(Subcommand)]
enum LedTest {
    /// Turn all LEDs on
    All,
    /// Turn all LEDs off
    Off,
    /// Sequential test: turn each LED on for 250ms, then off
    Seq,
    /// Random test: turn random LEDs on/off for 200 iterations
    Random,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Test { component } => {
            run_test(component).await?;
        }
        Commands::Server { port, host } => {
            run_server(port, host).await?;
        }
    }

    Ok(())
}

async fn run_test(component: TestComponent) -> Result<(), Box<dyn std::error::Error>> {
    println!("Train Set Control System - Test Mode");
    println!("Initializing LED controller...");

    // Initialize LED controller (24 LEDs on GPIO pins 4-27)
    let leds = LedController::new()?;
    println!("LED controller initialized with {} LEDs (GPIO pins 4-27)", leds.count());
    println!("  Green LEDs: 1-6");
    println!("  Amber LEDs: 7-12");
    println!("  Red LEDs: 13-24\n");

    match component {
        TestComponent::Led { test } => test_leds(leds, test).await?,
    }

    Ok(())
}

async fn test_leds(leds: LedController, test: LedTest) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== LED Test ===");

    match test {
        LedTest::All => {
            println!("Turning all LEDs on...");
            for led in 1..=24 {
                leds.on(led).await?;
            }
            println!("All {} LEDs are now ON", leds.count());
            println!("\nPress Enter to turn all LEDs off...");
            let mut buffer = String::new();
            std::io::stdin().read_line(&mut buffer)?;
            leds.all_off().await?;
            println!("All LEDs turned off");
        }
        LedTest::Off => {
            println!("Turning all LEDs off...");
            leds.all_off().await?;
            println!("All {} LEDs are now OFF", leds.count());
        }
        LedTest::Seq => {
            println!("Sequential LED test - turning each LED on for 250ms...");
            for led in 1..=24 {
                leds.on(led).await?;
                println!("  LED {}: ON", led);
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                leds.off(led).await?;
                println!("  LED {}: OFF", led);
            }
            println!("\nSequential test complete!");
        }
        LedTest::Random => {
            use rand::Rng;
            println!("Random LED test - 200 iterations...");
            let mut rng = rand::thread_rng();
            
            for iteration in 1..=200 {
                let random_led = rng.gen_range(1..=24);
                leds.on(random_led).await?;
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                leds.off(random_led).await?;
                
                if iteration % 20 == 0 {
                    println!("  Completed {} iterations...", iteration);
                }
            }
            println!("\nRandom test complete! (200 iterations)");
        }
    }

    Ok(())
}

async fn run_server(port: u16, host: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("Train Set Control System - Web Server Mode");
    println!("Initializing LED controller...");

    // Initialize LED controller (24 LEDs on GPIO pins 4-27)
    let leds = std::sync::Arc::new(LedController::new()?);
    println!("LED controller initialized with {} LEDs", leds.count());
    println!("  Green LEDs: 1-6");
    println!("  Amber LEDs: 7-12");
    println!("  Red LEDs: 13-24");

    // Create application state
    let app_state = AppState {
        leds,
    };

    // Create router
    let app = create_router(app_state);

    // Start server
    let addr = format!("{}:{}", host, port);
    println!("\nStarting web server on http://{}", addr);
    println!("API endpoints available at http://{}/api", addr);
    
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
