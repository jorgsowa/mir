===description===
A bare docblock return type named like a global builtin must resolve to a
same-namespace class when that class exists, including at a method call site.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedMethod errorLevel="suppress"/>
    <UnusedProperty errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace RandomLib;

final class Generator {}

final class Factory {
    /** @return Generator */
    public function getHighStrengthGenerator() {
        return new Generator();
    }
}

final class Adapter {
    private Generator $generator;

    public function __construct(Factory $factory) {
        $this->generator = $factory->getHighStrengthGenerator();
    }
}
===expect===
