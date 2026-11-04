===description===
`object{...}` keeps its property types: reading a declared property yields
its type, and a shape argument must have the parameter shape's properties.
Any class instance or plain `object` is accepted, since its properties are
checked only against another shape.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param object{id: int, name?: string} $o */
function idOf(object $o): string {
    return $o->id;
//  ^^^^^^^^^^^^^^ InvalidReturnType: Return type 'int' is not compatible with declared 'string'
}

/** @param object{id: int, name?: string} $o */
function nameOf(object $o): ?string {
    return $o->name;
}

/** @param object{id: int} $o */
function takesId(object $o): void {}

/**
 * @param object{id: int, extra: bool} $wider
 * @param object{id: string} $wrongType
 * @param object{name: string} $missing
 */
function callers(object $wider, object $wrongType, object $missing, object $plain, \stdClass $named): void {
    takesId($wider);
    takesId($wrongType);
//          ^^^^^^^^^^ InvalidArgument: Argument $o of takesId() expects 'object{id: int}', got 'object{id: string}'
    takesId($missing);
//          ^^^^^^^^ InvalidArgument: Argument $o of takesId() expects 'object{id: int}', got 'object{name: string}'
    takesId($plain);
    takesId($named);
}

/** @return object{id: int} */
function make(): object {
    return 1;
//  ^^^^^^^^^ InvalidReturnType: Return type '1' is not compatible with declared 'object{id: int}'
}
