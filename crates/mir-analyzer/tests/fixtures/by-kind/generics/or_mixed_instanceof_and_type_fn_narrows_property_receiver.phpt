===description===
Property-access counterpart of or_mixed_instanceof_and_type_fn_narrows_to_union
— `$this->prop instanceof A || is_string($this->prop)` narrows the property
to A|string. Pure-instanceof-OR and pure-type-fn-OR property narrowing both
already existed; the mixed-kind combination didn't.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {}

final class Holder {
    /** @var A|string|int */
    public $prop;

    public function narrows(): void {
        if ($this->prop instanceof A || is_string($this->prop)) {
            /** @mir-check $this->prop is A|string */
            $_ = 1;
        }
    }
}
