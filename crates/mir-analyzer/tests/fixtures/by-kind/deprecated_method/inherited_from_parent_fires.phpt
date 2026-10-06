===description===
DeprecatedMethod fires when calling a deprecated parent method on a child instance that does not override it.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {
    /** @deprecated use newMethod() instead */
    public function oldMethod(): void {}
}
class Child extends Base {}

function test(Child $c): void {
    $c->oldMethod();
//  ^^^^^^^^^^^^^^^ DeprecatedMethod: Method Child::oldMethod() is deprecated: use newMethod() instead
}
