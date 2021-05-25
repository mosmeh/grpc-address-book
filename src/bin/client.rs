use grpc_address_book::address_book::{address_book_client::AddressBookClient, *};
use structopt::StructOpt;
use tonic::{transport::Channel, Request};

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long, env = "SERVER_ADDR")]
    addr: String,

    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
enum Command {
    Create {
        #[structopt(short, long)]
        name: String,
        #[structopt(short, long)]
        email: Option<String>,
    },
    Get {
        uuid: String,
    },
    List,
    Search {
        #[structopt(short, long)]
        name: Option<String>,
        #[structopt(short, long)]
        email: Option<String>,
    },
    Update {
        uuid: String,
        #[structopt(short, long)]
        name: Option<String>,
        #[structopt(short, long)]
        email: Option<String>,
    },
    Delete {
        uuid: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let opt = Opt::from_args();

    let addr = format!("http://{}", opt.addr).parse()?;
    let client = Channel::builder(addr).connect().await?;
    let mut client = AddressBookClient::new(client);

    match opt.command {
        Command::Create { name, email } => {
            let response = client
                .create_person(Request::new(CreatePersonRequest { name, email }))
                .await?;
            println!("{}", response.into_inner().uuid);
        }
        Command::Get { uuid } => {
            let response = client
                .get_person(Request::new(GetPersonRequest { uuid }))
                .await?;
            println!("{:#?}", response.into_inner().person.unwrap());
        }
        Command::List => {
            let response = client.list_people(Request::new(())).await?;
            println!("{:#?}", response.into_inner().people);
        }
        Command::Search { name, email } => {
            let response = client
                .search_people(Request::new(SearchPeopleRequest { name, email }))
                .await?;
            println!("{:#?}", response.into_inner().people);
        }
        Command::Update { uuid, name, email } => {
            let _ = client
                .update_person(Request::new(UpdatePersonRequest { uuid, name, email }))
                .await?;
        }
        Command::Delete { uuid } => {
            let _ = client
                .delete_person(Request::new(DeletePersonRequest { uuid }))
                .await?;
        }
    }

    Ok(())
}
