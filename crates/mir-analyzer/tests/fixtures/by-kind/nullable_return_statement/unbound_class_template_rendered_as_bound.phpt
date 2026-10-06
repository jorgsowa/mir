===description===
A bare reference to a generic class leaves its template unbound, but the
message renders it as its bound rather than the raw template name.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Shape {}

/** @template T of Shape */
final class Bounded {
    /** @return T|null */
    public function find() { return null; }
}

final class Holder {
    private Bounded $bounded;

    public function viaMethod(): Shape {
        return $this->bounded->find();
    }

    public function viaClosure(): Closure {
        return function (): Shape {
            return $this->bounded->find();
        };
    }
}
===expect===
NullableReturnStatement@14:8-14:38: Return type 'Shape|null' is not compatible with declared 'Shape'
NullableReturnStatement@19:12-19:42: Return type 'Shape|null' is not compatible with declared 'Shape'
