===description===
`($this->prop ?? FALLBACK) === FALLBACK` narrows `$this->prop` to non-null on
the false branch, the same way the plain-variable form already does.
===config===
suppress=UnusedVariable,MissingConstructor
===file===
<?php
final class Bag {
    /** @var ?string */
    public ?string $label = null;

    public function narrowsOnFalseBranch(): void {
        if (($this->label ?? 'default') !== 'default') {
            /** @mir-check $this->label is string */
            $_ = 1;
        }
    }

    public function reversedOperands(): void {
        if ('default' !== ($this->label ?? 'default')) {
            /** @mir-check $this->label is string */
            $_ = 1;
        }
    }

    public function trueBranchLeavesNullable(): void {
        if (($this->label ?? 'default') === 'default') {
            /** @mir-check $this->label is ?string */
            $_ = 1;
        }
    }
}
===expect===
