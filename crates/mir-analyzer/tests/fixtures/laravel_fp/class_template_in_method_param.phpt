===description===
Laravel FP: a class-level `@template TRelatedModel` referenced in a method
`@param Collection<int, TRelatedModel> $results` was wrongly namespace-qualified
to `Illuminate\Database\Eloquent\Relations\TRelatedModel` because
`build_method_storage` built its template-name set AFTER the param loop.
The fix moves template setup before the param loop and uses
`resolve_union_doc_with_templates` for method params, so class-level template
params are recognized and stored as `TTemplateParam`.
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,MissingPropertyType,MixedArgument,MixedAssignment,MixedReturnStatement,MixedMethodCall
===file===
<?php
namespace App\Relations;

/**
 * @template TRelatedModel of Model
 */
class HasOneOrMany {
    /**
     * @param \Illuminate\Database\Eloquent\Collection<int, TRelatedModel> $results
     */
    public function matchOneOrMany(array $models, $results): void {}
}

class Model {}
===expect===
UndefinedDocblockClass@11:20-11:34: Docblock type 'Illuminate\Database\Eloquent\Collection' does not exist
