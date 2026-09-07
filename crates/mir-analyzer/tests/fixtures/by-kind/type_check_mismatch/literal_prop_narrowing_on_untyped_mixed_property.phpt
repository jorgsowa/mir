===description===
`$this->prop === 'literal'` / `=== 42` / `=== EnumCase` must narrow an
untyped (declared-mixed) property the same way a plain variable already
does — narrow_prop_literal_string/_int/_to_literal_enum_case each had an
early `current.is_mixed()` bail that their var-receiver siblings (and the
sibling narrow_prop_to_class_string/_to_specific_class) don't have, even
though the narrowing logic itself already handles a TMixed atom correctly.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
enum Status {
    case Active;
    case Done;
}

class C {
    public $status;
    public $count;
    public $state;

    public function stringLiteral(): void {
        if ($this->status === 'active') {
            /** @mir-check $this->status is 'active' */
            $_ = 1;
        }
    }

    public function intLiteral(): void {
        if ($this->count === 5) {
            /** @mir-check $this->count is 5 */
            $_ = 1;
        }
    }

    public function enumCase(): void {
        if ($this->state === Status::Active) {
            /** @mir-check $this->state is Status::Active */
            $_ = 1;
        }
    }
}
===expect===
