import * as cdk from 'aws-cdk-lib';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as apigateway from 'aws-cdk-lib/aws-apigateway';
import { Construct } from 'constructs';

export class CdkDeployStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // Define the Lambda function
    const lambdaFunction = new lambda.Function(this, 'MerkleInfosHandler', {
      runtime: lambda.Runtime.PROVIDED,
      code: lambda.Code.fromAsset('lambda.zip'),
      handler: 'hello', // No importance here, whatever
    });

    // Create an API Gateway REST API
    const api = new apigateway.RestApi(this, 'MerkleInfosApi', {
      restApiName: 'MerkleInfosService',
    });

    // Create a resource and method for the API Gateway
    const merkleInfos = api.root.addResource('merkleinfos');
    merkleInfos.addMethod('GET', new apigateway.LambdaIntegration(lambdaFunction));
  }
}