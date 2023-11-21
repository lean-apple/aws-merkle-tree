#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import { CdkDeployStack } from '../lib/cdk-deploy-stack';

const app = new cdk.App();
new CdkDeployStack(app, 'CdkDeployStack', {

  /* Uncomment the next line if you know exactly what Account and Region you
   * want to deploy the stack to. */
  env: { account: "826607129737", region: "eu-west-3" }

});