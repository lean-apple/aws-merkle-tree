## Merkle Tree Data Store and API

**Project Overview**
This project aims to create a binary Merkle tree data store and expose it through a simple API route.

- The Merkle tree and especially their node leaves will be persisted using AWS DynamoDB, and the API route will be implemented using AWS Lambda and the [AWS CDK - Cloud Development Kit](https://aws.amazon.com/fr/cdk/).

- [Amazon DynamoDB](https://aws.amazon.com/fr/dynamodb) was picked as a storage mechanism as it provides a highly scalable and fully managed NoSQL database service, making it suitable for efficiently storing and retrieving the Merkle tree data in a distributed and serverless environment.

- The Merkle Tree was initialize in DynamoDB with 8 data leave to build a Binary Merkle Tree of 15 leaves.

Only the method  `get_node_info_from_db` is exposed. 
It enables to get the node infos from requetsing the index. 

### Prerequisites

Before any first step, ensure you have the following prerequisites :

- Rust installed (https://www.rust-lang.org/tools/install) 
- AWS CDK installed (https://docs.aws.amazon.com/cdk/latest/guide/getting_started.html)

### Compile the Merkle Tree Implementation

1. Build the binary 

```bash
cd merkle
cargo build --release --target x86_64-unknown-linux-musl
```

Notice the target `x86_64-unknown-linux-musl` was specified. This target is important because AWS Lambda runs on Amazon Linux, a Linux-based environment as explicited. In that way, the resulting binary is compatible with AWS Lambda's execution environment.

2. Tests

Basic tests are available at `/tests/tests.rs`, they can be checked them :

```bash
cd merkle
cargo test
```

### Deploy the Merkle Tree with AWS CDK

The merkle tree code was deployed through [AWS CDK](https://aws.amazon.com/fr/cdk/). 

Once the build of the Merkle Tree is done, it was needed to copy the merkle binary : 

```bash
cp merkle/target/x86_64-unknown-linux-musl/release/merkle cdk-deploy/
```

This step is necessary because AWS CDK and AWS Lambda natively support languages like Typescript, Python, and Java, but Rust is not natively supported. By copying the Rust binary into the deployment directory, we ensure that it can be executed within the Lambda function environment.

It is needed then to zip it. 

A global script is available at the path `/merkle/build.sh` to enable the right copying of the file and its formatting.

1. Bootstrapping

Before deploying, make sure you have the AWS CLI configured with the necessary credentials and CDK bootstrapped.

- Example of cdk bootstrapping 

```bash
cd cdk-deploy
cdk bootstrap aws://ACCOUNT-NUMBER/REGION
```

2. Deployment

- Use the AWS CDK to deploy the AWS Lambda function that will execute the Merkle tree binary

```bash
cd cdk-deploy
cdk deploy
```
It will deploy the Lambda function and create the necessary AWS resources.

3. Other way of deploying Rust code on Lambda without using CDK 

It is posible to deploy the compiled binary to AWS Lambda with [`cargo lambda`](https://www.cargo-lambda.info/). 

`cargo lambda build` and `cargo lambda deploy` can be used in that way from the `merkle` folder to directly deploy on AWS lambda, assuming aws config is already set up. 


### Testing the API

Still with the assumption that you have the AWS CLI configured with the necessary credentials to be able to request DynamoDB table : 

```bash 
curl "https://[api-gateway-id].execute-api.[region].amazonaws.com/prod/merkleinfos?index=[node-index]"
```
It will return this type of result which are the node infos for a specific index, here for the index 4 : 
```bash
{"depth":2,"hash":"f9b5e44bf841fa0154502b136be13274027480e4476595cc3c008c035c335501","offset":1}
```

It is sometimes needed to complete the `/cdk-deploy/lib/cdk-deploy-stack.ts` config in that way to give access to the user to the teh right to request dynamoDb. 


```ts
    // Define a policy statement that grants access to DynamoDB
    const dynamoDbPolicy = new iam.PolicyStatement({
      actions: ['dynamodb:GetItem'],
      resources: ['arn:aws:dynamodb:[region]:[user-id]:table/DevMerkleTree'],
    });

    // Attach the policy to the Lambda function's execution role
    lambdaFunction.role?.attachInlinePolicy(
      new iam.Policy(this, 'DynamoDbAccessPolicy', {
        statements: [dynamoDbPolicy],
      }),
    );

```

It can be also done directly through IAM policies part of AWS' console.

