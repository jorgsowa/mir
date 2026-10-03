===description===
callable return type resolved
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template T
 */
class Registry {
    /** @return callable(): T */
    public function resolver(): callable { return function() { return null; }; }
}
class Service { public function handle(): void {} }
function test(): void {
    /** @var Registry<Service> $reg */
    $reg = new Registry();
    $resolver = $reg->resolver();
    /** @mir-check $resolver is callable():Service */
    $resolver()->undefinedMethod();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Service::undefinedMethod() does not exist
}
===expect===
