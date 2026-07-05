===description===
G1: `@implements Target<Arg>` / `@extends Target<Arg>` type args must be
checked against Target's own declared `@template T of Bound` — this was
previously never checked anywhere (only call-site/constructor-site bindings
were), so a class could declare a bound-violating type arg with no diagnostic.
===config===
suppress=UnusedVariable,MissingReturnType,MissingConstructor
===file===
<?php
class Base {}
class Unrelated extends Base {}
class NotBase {}

/** @template T of Base */
interface Container {}

/** @template T of Base */
class AbstractBox {}

// Satisfies the bound — no error.
class OkBag implements Container {}
/** @implements Container<Unrelated> */
class OkBag2 implements Container {}
/** @extends AbstractBox<Unrelated> */
class OkBox extends AbstractBox {}

// Violates the bound — NotBase does not extend Base.
/** @implements Container<NotBase> */
class BadBag implements Container {}
/** @extends AbstractBox<NotBase> */
class BadBox extends AbstractBox {}
===expect===
InvalidTemplateParam@21:0-21:36: Template type 'T' inferred as 'NotBase' does not satisfy bound 'Base'
InvalidTemplateParam@23:0-23:35: Template type 'T' inferred as 'NotBase' does not satisfy bound 'Base'
