===description===
Laravel FP: `@return static|void` on methods like `Password::defaults()`,
`Email::defaults()`, and `File::defaults()`. These have a guarded `return static::...`
branch and an implicit void fall-through branch. `return_requires_value()` only
checked `t.is_void()` (pure single-atomic void), so it fired `InvalidReturnType`
for `void` even when `void` is a union member. Fixed by adding a
`t.contains(TVoid)` guard.
===config===
suppress=MissingPropertyType,MissingClosureReturnType,UnusedParam,UnusedVariable,MixedMethodCall,MixedReturnStatement,InvalidPropertyAssignment
===file===
<?php
class Password {
    /** @var callable|null */
    public static $defaultCallback = null;

    /**
     * @param  static|callable|null  $callback
     * @return static|void
     */
    public static function defaults($callback = null) {
        if (is_null($callback)) {
            return new static();
        }
        static::$defaultCallback = $callback;
    }
}
===expect===
