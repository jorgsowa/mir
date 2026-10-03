===description===
Same as the interface case, through an abstract parent class and a transitive
`@extends` binding.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
abstract class Maker {
    /** @return T */
    abstract public function make(): Maker;
}

/** @extends Maker<Widget> */
abstract class WidgetMaker extends Maker {}

final class Widget extends WidgetMaker {
    public function make(): Maker {
        return new self();
    }
}
===expect===
