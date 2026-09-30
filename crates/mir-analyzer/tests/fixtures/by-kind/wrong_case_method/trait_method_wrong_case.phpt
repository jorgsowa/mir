===description===
Wrong case method name provided by a trait is reported.
===file===
<?php
trait Serializable2 {
    public function toJson(): string { return "{}"; }
}
class Model {
    use Serializable2;
}
$m = new Model();
$m->TOJSON();
//  ^^^^^^ WrongCaseMethod: Method name 'Model::TOJSON' has incorrect casing; use 'toJson'
===expect===
