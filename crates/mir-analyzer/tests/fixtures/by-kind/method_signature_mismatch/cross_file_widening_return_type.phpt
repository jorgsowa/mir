===description===
cross file widening return type
===file:Base.php===
<?php
class Base {
    public function fetch(): string { return ""; }
}
===file:Child.php===
<?php
class Child extends Base {
    public function fetch(): ?string { return null; }
}
===expect===
Child.php: MethodSignatureMismatch@3:4-3:53: Method Child::fetch() signature mismatch: return type 'string|null' is not a subtype of parent 'string'
