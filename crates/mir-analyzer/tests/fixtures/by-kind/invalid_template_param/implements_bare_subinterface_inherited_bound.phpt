===description===
`@implements Target<Arg>` bound-checking must resolve Target's inherited
`@template T of Bound` even when Target itself is a bare sub-interface
that doesn't redeclare `@template` — previously `check_generic_type_args`
looked up Target's own-only template params, so this silently passed.
===config===
suppress=UnusedVariable,MissingReturnType,MissingConstructor
===file===
<?php
/** @template T of Countable */
interface Repository {}

interface MidRepository extends Repository {}

class GoodCounter implements Countable {
    public function count(): int { return 0; }
}

/** @implements MidRepository<GoodCounter> */
class OkRepo implements MidRepository {}

/** @implements MidRepository<stdClass> */
class BadRepo implements MidRepository {}
===expect===
InvalidTemplateParam@15:0-15:41: Template type 'T' inferred as 'stdClass' does not satisfy bound 'Countable'
