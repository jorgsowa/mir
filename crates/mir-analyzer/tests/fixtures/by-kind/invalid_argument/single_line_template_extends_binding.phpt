===description===
@template and @extends packed onto one docblock line both still parse and bind
===file===
<?php
/** @template V */
class GrandParentBox {
    /** @param V $v @return V */
    public function process($v) {
        return $v;
    }
}

/** @template U @extends GrandParentBox<U> */
class ChildBox extends GrandParentBox {}

/** @extends ChildBox<int> */
class GrandChildBox extends ChildBox {}

$gc = new GrandChildBox();
$gc->process("this is a string, not an int");
//           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $v of process() expects 'int', got '"this is a string, not an int"'
