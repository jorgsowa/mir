===description===
An interface's own `@template-extends Base<Arg>` type args were parsed and
used for binding substitution elsewhere, but never bound-checked here — only
the class-level `@implements`/`@extends` counterpart was (see
implements_extends_check_template_bound.phpt).
===config===
suppress=UnusedVariable,MissingReturnType,MissingConstructor
===file===
<?php
class Base {}
class Unrelated extends Base {}
class NotBase {}

/** @template T of Base */
interface Container {}

// Satisfies the bound — no error.
/** @template-extends Container<Unrelated> */
interface OkChild extends Container {}

// Violates the bound — NotBase does not extend Base.
/** @template-extends Container<NotBase> */
interface BadChild extends Container {}
===expect===
InvalidTemplateParam@15:0-15:39: Template type 'T' inferred as 'NotBase' does not satisfy bound 'Base'
