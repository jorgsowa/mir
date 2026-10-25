===description===
A concrete class implementing Traversable, directly or through an interface, must also implement Iterator or IteratorAggregate; abstract classes and subclasses of an iterable parent are fine.
===file===
<?php
interface Collection extends Traversable {}

class Direct implements Traversable {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Class Direct must implement interface Traversable as part of either Iterator or IteratorAggregate [traversable-without-iterator]

class ViaInterface implements Collection {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Class ViaInterface must implement interface Traversable as part of either Iterator or IteratorAggregate [traversable-without-iterator]

abstract class Base implements Traversable {}

class Items implements Collection, IteratorAggregate {
    public function getIterator(): Iterator { return new ArrayIterator([]); }
}

class Period extends DatePeriod {}

class More extends Items {}
