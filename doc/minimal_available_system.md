Minimal Available System Test on Google Cloud
---

### VPC for Serverless

Update: The easiest way to do it is to deploy Cloud SQl and Compute Engine's private IP section as default and no need to create an individual VPC.

Create a VPC network:   https://console.cloud.google.com/networking/networks/list


    name: insta-share-vpc1
    subnet: custom
    subnet name: insta-share-vpc-uw1
    region: us-central1
    ip: 192.168.0.0/24
    private google accss: on

Firewall rules:

+ delete 'allow rdp'

+ add 'allow internal communication' ? (not done yet)

Egress must have an external IP address, so it's not needed in many internal services.

Router:

Not configured yet.

Serverless VPC Connector:

+ Enable 'Serverless VPC Access API'

+ Create connector in VPC page.

    name: insta-share-vpc-connector
    region: us-central1 (not many options)
    ip: 10.8.0.0/28 (default)

---

### Cloud SQL

+ Create a postgres database in GCP console.

By the way, the default config is 1 vcpu, 3.75G RAM and 20G SSD.

+ Edit instance, enable private IP connection

    private IP: 10.214.144.3
    anno-square:us-central1:test-psql

Public could be disabled later for security.

+ Install postgres client on machine and connect to database in gcloud sdk tool:

    gcloud sql connect instance-name --user=postgres

Now create database, table, and populate some data into it.

Build rust program in release mode and upload it to a cloud instance for test.

### Secret Manager

+ Store the database password to Secreat manager.

+ IAM -> Service account -> Create service account -> Give secret access permission to a role.

+ Create an Compute Engine instance of this role. Now it has permission to retrieve secret by:

    curl "https://secretmanager.googleapis.com/v1/projects/`project-id`/secrets/`secret-id`/versions/`version-id`:access" \
        --request "GET" \
        --header "authorization: Bearer $(gcloud auth print-access-token)" \
        --header "content-type: application/json" \
        --header "x-goog-user-project: `project-id`"

Note the response should be decoded by base64.

### Communication between Cloud Run and Cloud SQL in Rust

Start a postgres container from docker-compose.yml

    version: '3'
    services:
    db:
        image: postgres:latest
        container_name: db
        restart: "no"
        ports:
        - "5432:5432"
        environment:
        - POSTGRES_USER=admin
        - POSTGRES_PASSWORD=1a2b3c
        - POSTGRES_DB=test
        volumes:
        - ./pgdata:/var/lib/postgresql/data
        - ./pginit:/docker-entrypoint-initdb.d/


Then step into it by :
    docker exec -it db psql -U admin test

or:
    docker exec -it bash

followed by `psql -U admin test`.


Local code: rust and postgresql

https://cloud.google.com/vpc/docs/configure-serverless-vpc-access