===description===
Laravel FP: `Container::make('files')` and similar IoC binding-key strings
emit `UndefinedClass`. The parameter type is `string|class-string<TClass>`,
so a literal string satisfies the `string` alternative and is always valid —
no class-existence check required. Fixed by skipping `validate_class_string_argument`
when the parameter also accepts a plain `string`.
===config===
suppress=MissingPropertyType,MissingClosureReturnType,UnusedParam,UnusedVariable,MixedMethodCall,MixedArgument,MixedReturnStatement,UndefinedMethod,MixedAssignment
===file===
<?php

/**
 * @template TClass of object
 */
class Container {
    /**
     * @template TClass of object
     * @param string|class-string<TClass> $abstract
     * @return ($abstract is class-string<TClass> ? TClass : mixed)
     */
    public static function make(string $abstract): mixed {
        return new stdClass();
    }
}

class ServiceProvider {
    public function register(): void {
        // These IoC binding keys are plain strings, not class names.
        // With string|class-string<T> param, they satisfy `string` — no UndefinedClass.
        $files = Container::make('files');
        $config = Container::make('config');
        $session = Container::make('session');
        $bladeCompiler = Container::make('blade.compiler');
    }
}
===expect===
