export type Account = {
    id: string,
    name: string,
    alias: string,
    owner: string,
    kind: AccountKind
};

export type AccountKind = 'NRA' | 'TFSA' | 'RRSP' | 'FHSA'

export const kindOptions = [
    'NRA',
    'TFSA',
    'RRSP',
    'FHSA'
]
