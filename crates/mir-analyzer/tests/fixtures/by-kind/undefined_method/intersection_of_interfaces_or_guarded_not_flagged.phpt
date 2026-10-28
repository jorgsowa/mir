===description===
An intersection call is checked like a call on each part: interfaces and abstract classes do not prove a method missing, nor does a part behind a `method_exists()` guard
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface A { public function a(): int; }
interface B { public function b(): int; }
abstract class Base {}
class Plain {}

final class User {
    public function __construct(
        private A&B $ab,
        private Base&A $baseA,
        private Plain&A $plainA,
    ) {}

    public function run(): void {
        $this->ab->undeclared();
        $this->baseA->undeclared();
        if (method_exists($this->plainA, 'reset')) {
            $this->plainA->reset();
        }
        $this->plainA->reset();
//      ^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Plain&A::reset() does not exist
    }
}
