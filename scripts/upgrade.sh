#!/bin/bash
dfx canister --ic install codegov-wtn-vote-relay --mode upgrade --upgrade-unchanged --argument '(variant { Upgrade = record {}})'