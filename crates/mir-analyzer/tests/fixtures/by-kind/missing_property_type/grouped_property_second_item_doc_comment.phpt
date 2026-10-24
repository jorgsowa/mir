===description===
Each item of a grouped property declaration keeps its own doc comment: the @var on the second item types it.
===file===
<?php
class A {
    /** @var int */
    public $a = 1, /** @var string */ $b = "x";
    public function b(): string { return $this->b; }
    public function a(): string { return $this->a; }
//                                ^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'int' is not compatible with declared 'string'
}
