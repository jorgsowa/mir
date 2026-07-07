===description===
Narrowing a class-level @template-typed PROPERTY via instanceof must apply
the same T&Class intersection bfbb69f4 already added for local variables.
narrow_prop_instanceof kept its own is_mixed() guard (missed by that
audit), so an unconstrained property template was treated as real mixed
and the instanceof narrowing was skipped entirely, leaving the property at
its bare, unrefined declared type instead of narrowing it to T&Countable.
===config===
suppress=UnusedVariable,MissingReturnType,MixedArgument,MissingPropertyType
===file===
<?php
/**
 * @template T
 */
class Box {
    /** @var T */
    public $value;

    public function check(): void {
        if ($this->value instanceof Countable) {
            $local = $this->value;
            /** @mir-check $local is T&Countable */
            $_ = 1;
        }
    }
}
===expect===
