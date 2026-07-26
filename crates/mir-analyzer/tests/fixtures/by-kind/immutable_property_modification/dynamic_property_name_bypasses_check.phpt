===description===
A dynamic property name (`$this->$prop = x`, `$other->$prop = x`) silently
bypassed the immutable-write check entirely — `extract_string_from_expr`
returns None for a variable property name, and every call site guarded the
check behind `if let Some(prop_name) = ...`, so it quietly no-oped instead
of falling back to a conservative "assume mutation" treatment. Falls back
to the property expression's own source text as a display name.
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
/** @psalm-immutable */
class Box {
    public int $x = 0;

    public function mutate(string $prop): void {
        $this->$prop = 5;
    }
}

class Caller {
    public function mutateExternal(Box $b, string $prop): void {
        $b->$prop = 5;
    }
}
===expect===
ImmutablePropertyModification@7:8-7:24: Assigning to property $prop of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
ImmutablePropertyModification@13:8-13:21: Assigning to property $prop of $b in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
