===description===
A read of a property is reported once a method in another file starts storing
request data in it.
===file:Store.php===
<?php
class Store {
    public string $value = '';
    public function fill(): void { $this->value = 'fixed'; }
}
===file:Use.php===
<?php
function show(Store $store): void { echo $store->value; }
===expect===
<<none>>
===edit:Store.php===
<?php
class Store {
    public string $value = '';
    public function fill(): void { $this->value = (string) $_GET['v']; }
}
===expect===
Store.php: MixedArrayAccess@4:59-4:69: Array access on mixed type
Use.php: TaintedHtml@2:36-2:55: Tainted HTML output — possible XSS
