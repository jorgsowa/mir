===description===
Regression (laravel/framework): a `@return static<TValue, TKey>` annotation
referencing the class's own @template params must treat TKey/TValue as template
params, not resolve them to namespaced classes (Illuminate\Support\TKey). mir no
longer emits InvalidTemplateParam (Collection::flip).
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedProperty errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
===expect===
