===description===
bare generic PHP-typed property accepts parameterized actual where type param is built-in object type
===file===
<?php
/** @template T */
class Container {}

class Factory {
    /**
     * @return ($x is null ? Container<object> : Container<object>)
     */
    public function make($x): Container { return new Container(); }

    public Container $prop;
}

$f = new Factory();
$result = $f->make(null);
/** @mir-check $result is Container<object> */
$f->prop = $result;
===expect===
MissingConstructor@5:0-5:15: Class Factory has uninitialized properties but no constructor
UnusedParam@9:25-9:27: Parameter $x is never used
