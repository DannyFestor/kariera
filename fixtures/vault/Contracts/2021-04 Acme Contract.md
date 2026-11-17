---
type: contract
counterparty: "[[Acme]]"
contract_type: quasi-mandate
start: 2021-04-01
end: 2023-09-30
rate: 5000
rate_unit: hour
rate_history:
  - from: 2021-04
    rate: 4500
  - from: 2022-04
    rate: 5000
sources:
  - kind: contract
    path: .scratch/_inputs/contracts/acme/basic-agreement.pdf
    note: Basic agreement, renewed every six months
  - kind: contract
    path: .scratch/_inputs/contracts/acme/individual-2022-04.pdf
    note: Individual contract from 2022-04 at 5,000 JPY an hour
---
