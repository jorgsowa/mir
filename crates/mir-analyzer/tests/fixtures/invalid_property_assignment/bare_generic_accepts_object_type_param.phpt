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
$f->prop = $f->make(null);
===expect===
UnusedParam@9:26: Parameter $x is never used
