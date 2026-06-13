===description===
Laravel FP (laravel/framework): a bare `return;` yields null, which is assignable
to a nullable declared return type (?User). mir synthesizes `void` and emits
InvalidReturnType. Ignored pending fix — see ROADMAP §1.4 (stmt/flow.rs bare-return
guard ignores nullable/void-union declared types).
===ignore===
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedFunction,MixedReturnStatement
===file===
<?php
class User {}
class EloquentUserProvider {
    public function retrieveByToken(int $id): ?User {
        if ($id < 0) {
            return;
        }
        return new User();
    }
}
===expect===
