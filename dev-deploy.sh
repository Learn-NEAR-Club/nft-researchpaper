set -e

cd contract && ./build.sh && cd ../ && rm -rf neardev
near dev-deploy || exit 0 && source ./neardev/dev-account.env

acc_name="${CONTRACT_NAME}.testnet"

near call $CONTRACT_NAME new_standard --accountId=$CONTRACT_NAME

near call $CONTRACT_NAME submit '{"token_id": "0", 
    "title": "Bitcoin: A Peer-to-Peer Electronic Cash System",
    "author": ["Nakamoto, Satoshi"],
    "accrev": ["nearlap2.nearlap.testnet","nearlap3.nearlap.testnet","nearlap4.nearlap.testnet"]}' --accountId=$CONTRACT_NAME --deposit=10

near call $CONTRACT_NAME stataccept '{"token_id": "0", "approv": "Approved"}' --accountId=nearlap2.nearlap.testnet
near call $CONTRACT_NAME voting '{"token_id": "0", "vote": "Yes"}' --accountId=nearlap2.nearlap.testnet

near call $CONTRACT_NAME stataccept '{"token_id": "0", "approv": "Approved"}' --accountId=nearlap3.nearlap.testnet
near call $CONTRACT_NAME voting '{"token_id": "0", "vote": "Yes"}' --accountId=nearlap3.nearlap.testnet

near call $CONTRACT_NAME stataccept '{"token_id": "0", "approv": "Approved"}' --accountId=nearlap4.nearlap.testnet
near call $CONTRACT_NAME voting '{"token_id": "0", "vote": "Yes"}' --accountId=nearlap4.nearlap.testnet

near call $CONTRACT_NAME payreviewer '{"token_id": "0"}' --accountId=$CONTRACT_NAME

near call $CONTRACT_NAME publish '{
    "token_id": "0", "receiver_id": "'${acc_name}'", "token_metadata": {
        "title": "Bitcoin: A Peer-to-Peer Electronic Cash System",
        "description": "Article",
        "media": null,
        "media_hash": null,
        "copies": null,
        "issued_at": null,
        "expires_at": null,
        "starts_at": null,
        "updated_at": null,
        "extra": null,
        "reference": null,
        "reference_hash": null}
}' --accountId=$CONTRACT_NAME --deposit=1

near view $CONTRACT_NAME view_papers --accountId=$CONTRACT_NAME
near view $CONTRACT_NAME view_paper_meta '{"token_id": "0"}' --accountId=$CONTRACT_NAME
