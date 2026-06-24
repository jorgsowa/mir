===description===
The constructor of a @psalm-immutable class is allowed to assign to $this->prop —
initialization is not a mutation.
===config===
suppress=MissingPropertyType
===file===
<?php

/** @psalm-immutable */
class Money {
    public int $amount;
    public string $currency;

    public function __construct(int $amount, string $currency) {
        $this->amount = $amount;
        $this->currency = $currency;
    }

    public function getAmount(): int {
        return $this->amount;
    }
}
===expect===
