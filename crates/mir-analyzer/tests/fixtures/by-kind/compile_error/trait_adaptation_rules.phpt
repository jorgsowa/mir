===description===
PHP checks every `insteadof` and `as` rule of a class-like: a named trait must exist, be a trait and be used; a precedence rule must name a method of its trait and not exclude that trait; an unqualified alias may not name a method two used traits declare.
===file===
<?php
trait Hello {
    public function greet(): string { return 'hello'; }
}

trait Hi {
    public function greet(): string { return 'hi'; }
}

trait Unused {
    public function greet(): string { return 'unused'; }
}

class NotATrait {}

class UnknownTrait {
    use Hello {
        Missing::greet as hey;
//      ^^^^^^^ UndefinedTrait: Trait Missing does not exist
    }
}

class UsesClass {
    use Hello, Hi {
        Hello::greet insteadof Hi;
        NotATrait::greet as hey;
//      ^^^^^^^^^ CompileError: Class NotATrait is not a trait, Only traits may be used in 'as' and 'insteadof' statements [adaptation-non-trait]
    }
}

class NotUsed {
    use Hello {
        Hello::greet insteadof Unused;
//                             ^^^^^^ CompileError: Required Trait Unused wasn't added to NotUsed [trait-not-used]
    }
}

class NoSuchMethod {
    use Hello, Hi {
        Hello::wave insteadof Hi;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: A precedence rule was defined for Hello::wave but this method does not exist [undefined-precedence-method]
        Hi::greet insteadof Hello;
    }
}

class ExcludesItself {
    use Hello, Hi {
        Hello::greet insteadof Hello, Hi;
//                             ^^^^^ CompileError: Inconsistent insteadof definition. The method greet is to be used from Hello, but Hello is also on the exclude list [inconsistent-insteadof]
    }
}

class AmbiguousAlias {
    use Hello, Hi {
        Hello::greet insteadof Hi;
        greet as salute;
//      ^^^^^^^^^^^^^^^^ CompileError: An alias was defined for method greet(), which exists in both Hello and Hi. Use Hello::greet or Hi::greet to resolve the ambiguity [ambiguous-trait-alias]
    }
}

trait Composed {
    use Hello, Hi {
        Hello::greet insteadof Hi;
        Hi::greet as hiGreet;
    }
}
