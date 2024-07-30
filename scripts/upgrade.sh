#!/bin/bash
dfx build --ic codegov-wtn-vote-relay || exit 1
dfx canister --ic install codegov-wtn-vote-relay --mode upgrade --upgrade-unchanged --argument '(variant { Upgrade = record {}})'