export type Transaction = {
    id: string,
    account: string,
    date: string,
    action: TxnAction,
};

export type TxnAction = TxnActionDep | TxnActionBuy

export type TxnActionDep = {
    type: TxnActionDepType,
    value: [number, string],
    fee: [number, string],
}
export type TxnActionDepType = "Deposit" | "Withdrawal"

export type TxnActionBuy = {
    type: TxnActionBuyType
    asset: [number, string],
    cash: [number, string],
    fee: [number, string],
}

export type TxnActionBuyType = "Buy" | "Sell"

export const actionOptions = [
    'Deposit',
    'Withdrawal',
    'Buy',
    'Sell',
]

// export function to_content(value: TxnAction): string {
//     if (value.type == "Deposit") {
//         let unit = value.value[1].split(':')[1]
//         return `Deposit ${value.value[0]} ${unit}`
//     } else if (value.type == "Withdrawal") {
//         let unit = value.value[1].split(':')[1]
//         return `Withdrawal ${value.value[0]} ${unit}`
//     } else if (value.type == "Buy") {
//         return `Buy ${value.asset[0]} ${value.asset[1]} @ x xxx per share`
//     } else if (value.type == "Interest") {
//         let unit = value.value[1].split(':')[1]
//         return `Earn ${value.value[0]} ${unit} interest`
//     }
// }
