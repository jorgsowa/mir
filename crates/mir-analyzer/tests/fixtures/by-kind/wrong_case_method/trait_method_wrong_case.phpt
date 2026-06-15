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
===expect===
WrongCaseMethod@9:4-9:10: Method name 'Model::TOJSON' has incorrect casing; use 'toJson'
