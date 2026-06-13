===description===
Laravel FP (laravel/framework): a `@return static<TValue, TKey>` annotation
referencing the class's own @template params fails because mir does not register
template params in the generic scope — it resolves TKey to a namespaced class
(Illuminate\Support\TKey) and checks it against the bound, emitting
InvalidTemplateParam (Collection::flip). Ignored pending fix — see ROADMAP §1.4.
===ignore===
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedProperty,MixedArgument,MixedAssignment,MixedReturnStatement,MixedMethodCall
===file===
<?php
namespace App\Support;

/**
 * @template TKey of array-key
 * @template TValue
 */
class Collection {
    /** @var array<TKey, TValue> */
    protected array $items = [];

    /**
     * @return static<TValue, TKey>
     */
    public function flip(): static {
        return new static(array_flip($this->items));
    }
}
