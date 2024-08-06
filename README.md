# CodeGov-WTN-Vote-Relay

Below are the instructions for configuring the vote relay canister so your WTN neuron will follow any NNS neuron you
choose on the WTN proposal topic called "Vote for NNS Proposals". The code that performs the vote relay is provided by
CodeGov. You are welcome to register your NNS/WTN neuron pairs with the CodeGov vote relay canister as per the
instructions below. However, our repo is open source, so you are also welcome clone and modify the code to work for your
own canister. This vote relay canister will replicate the vote of any NNS neuron you choose (it can be your own neuron,
a known neuron, or any other neuron) to vote on the corresponding WTN proposal. In order to register a NNS/WTN neuron
pair, you must be able to set the vote relay canister ID as a hotkey for the WTN neuron. Hence, you must be able to
control the WTN neuron. The app provides prompts for setting up the pairs and it has error checking to verify that the
setup is successful. You can also list all neuron pairs to verify that your pair was successful. There is a deregister
command that can only be performed by the same principal that set the original configuration. That way nobody can
deregister your pair. It is recommended to set your WTN neuron ID as a Followee for the proposal topic "Vote for NNS
Proposals" in order to prevent your Followee for "All Non-Critical Topics" from voting before your preferred NNS neuron
votes for you on this topic.

## Prerequisites:
You must be able to execute commands in a command line terminal and you must have any version of DFX installed.

## Clone Vote Relay Repo:
If you can run command line and have DFX installed, then create a directory where you want to clone the git repo for 
the vote relay app. Change to that directory and run the following command...

`git clone https://github.com/CodeGov-org/codegov-wtn-vote-relay.git`

## Set Hotkey: 
Decide which NNS neuron ID and WTN neuron ID pair you want to configure for vote following. It can be any
NNS neuron you trust (including your own neuron or any known neuron). However, you must be able to control the WTN
neuron in order for this pairing to work because you must set the vote relay canister ID as a hotkey for your WTN
neuron. The CodeGov vote relay canister ID is e5khx-vyaaa-aaaar-qagja-cai. Set the hotkey for your WTN neuron before
completing the next step. Now is also a good time to set your WTN neuron ID as a Followee for the proposal topic "Vote
for NNS Proposals". If you don't do this, then is it possible that your Followee for "All Non-Critical Topics" may vote
before your preferred NNS neuron votes on this topic. Setting a neuron to follow itself on a specific topic is the only
way to ensure that your Followee selection for the All Topics catch all doesn't vote for you on that topic.

Here is the vote relay canister ID again for ease of copy...

`e5khx-vyaaa-aaaar-qagja-cai`

## Register_Neuron_Pair: 
Change directory in your command line terminal to the folder called codegov-wtn-vote-relay, which was created when you 
cloned the repo. Run the following command from within this folder and then follow the directions when prompted.

`dfx canister --ic call codegov-wtn-vote-relay register_neuron_pair`

Below is the example from when the CodeGov NNS / WTN neuron pair was registered...

```
This method requires arguments.

Enter a value for RegisterNeuronPairArgs : record { name : text; nns_neuron_id : nat64; wtn_neuron_id : blob }
Enter field name : text
✔ Enter a text (type :e to use editor) · CodeGov
Enter field nns_neuron_id : nat64
✔ Enter a nat64 · 2649066124191664356
Enter field wtn_neuron_id : blob
✔ Select a way to enter blob · hex
✔ Enter hex string · 203312480b4aeef877f393f376533f4fdcaa477412ca1ae83abd8e897c0e726f
Sending the following argument:
(
  record {
    name = "CodeGov";
    nns_neuron_id = 2_649_066_124_191_664_356 : nat64;
    wtn_neuron_id = blob "\20\33\12\48\0b\4a\ee\f8\77\f3\93\f3\76\53\3f\4f\dc\aa\47\74\12\ca\1a\e8\3a\bd\8e\89\7c\0e\72\6f";
  },
)

Do you want to send this message? [y/N]
y
(variant { Ok = 6_284_408_412_449_121_150 : nat64 })
```

When prompted to enter the WTN neuron ID, you will first be asked if you want to enter TEXT, HEX, or FILE. Select HEX.

If you failed to enter the hotkey, then the variant will show `Err=variant{NotPermittedToVote}`. This means the
registration failed.

## List_Neuron_Pairs: 
You can verify that your NNS / WTN neuron pair is registered by running the list_neuron_pairs command below. If you see 
your pair, then the registration was successful.

`dfx canister --ic call codegov-wtn-vote-relay list_neuron_pairs`

This is an example of the output of this command...

```
(
  vec {
    record {
      id = 3_076_454_463_670_727_740 : nat64;
      admin = principal "zahk7-wlrqq-e2kcg-erf5x-mnwu5-i7ux7-yn34n-6la6z-tter3-peiqj-iae";
      name = "WPB";
      nns_neuron_id = 8_269_903_528_373_981_952 : nat64;
      wtn_neuron_id = blob "\ec\a8\df\ca\d7\09\75\b5\23\2e\1c\55\da\50\e7\34\ac\22\6c\ba\b4\e2\b3\af\e6\24\97\5a\74\c4\82\34";
    };
    record {
      id = 6_284_408_412_449_121_150 : nat64;
      admin = principal "zahk7-wlrqq-e2kcg-erf5x-mnwu5-i7ux7-yn34n-6la6z-tter3-peiqj-iae";
      name = "CodeGov";
      nns_neuron_id = 2_649_066_124_191_664_356 : nat64;
      wtn_neuron_id = blob "\20\33\12\48\0b\4a\ee\f8\77\f3\93\f3\76\53\3f\4f\dc\aa\47\74\12\ca\1a\e8\3a\bd\8e\89\7c\0e\72\6f";
    };
    record {
      id = 17_475_894_244_464_236_238 : nat64;
      admin = principal "tu45y-p4p3d-b4gg4-gmyy3-rgweo-whsrq-fephi-vshrn-cipca-xdkri-pae";
      name = "CodeGov";
      nns_neuron_id = 2_649_066_124_191_664_356 : nat64;
      wtn_neuron_id = blob "\20\33\12\48\0b\4a\ee\f8\77\f3\93\f3\76\53\3f\4f\dc\aa\47\74\12\ca\1a\e8\3a\bd\8e\89\7c\0e\72\6f";
    };
  },
)
```

## Deregister_Neuron_Pair: 
If you want to delete a neuron pair that you added, then it can be done with the command below.

`dfx canister --ic call codegov-wtn-vote-relay deregister_neuron_pair`

You will be prompted for the registration ID that was created when you registered your pair. Your pair ID can be found
using the `list_neuron_pairs` command above. You must use the same principle ID to deregister that you used when you
register your pair. This ensures nobody else can deregister your NNS/WTN neuron pair.
