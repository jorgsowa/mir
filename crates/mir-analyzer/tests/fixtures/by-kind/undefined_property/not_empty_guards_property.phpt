===description===
`!empty($this->p)` proves an undeclared property is set, like `isset()`.
===file===
<?php
abstract class A {
    public function f(): string {
        return !empty($this->uuid) ? (string) $this->uuid : '';
    }
}
