===description===
An enum's `@implements Container<Arg>` type arg must be checked against
Container's own `@template T of Bound`, same as the identical class-level
check — the enum loop in class/mod.rs never called check_generic_type_args
for enum_def.implements_type_args.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {}
class Unrelated extends Base {}
class NotBase {}

/** @template T of Base */
interface Container {}

// Satisfies the bound — no error.
/** @implements Container<Unrelated> */
enum OkStatus implements Container { case Active; }

// Violates the bound — NotBase does not extend Base.
/** @implements Container<NotBase> */
enum BadStatus implements Container { case Active; }
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'T' inferred as 'NotBase' does not satisfy bound 'Base'
===expect===
