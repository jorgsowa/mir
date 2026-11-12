===description===
An array element passed by reference to a method, static method, function or constructor takes the type the parameter writes (its declared type, or `mixed` when untyped), so its old shape is dropped after the call. An element passed by value keeps its shape.
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

final class Builder
{
    public function __construct(&$form)
    {
        $form['#title'] = 'title';
    }

    public function build(&$form): void
    {
        $form['#title'] = 'title';
    }

    /** @param array $form */
    public function buildTyped(&$form): void
    {
        $form['#title'] = 'title';
    }

    public static function make(&$form): void
    {
        $form['#title'] = 'title';
    }
}

function fill(&$form): void
{
    $form['#title'] = 'title';
}

function viaMethod(Builder $builder): array
{
    $form = ['options' => ['#theme' => 'container']];
    $builder->build($form['options']);
    return [$form['options']['#title']];
}

function viaStatic(): array
{
    $form = ['options' => ['#theme' => 'container']];
    Builder::make($form['options']);
    return [$form['options']['#title']];
}

function viaFunction(): array
{
    $form = ['options' => ['#theme' => 'container']];
    fill($form['options']);
    return [$form['options']['#title']];
}

function viaConstructor(): array
{
    $form = ['options' => ['#theme' => 'container']];
    new Builder($form['options']);
    return [$form['options']['#title']];
}

function typedParam(Builder $builder): array
{
    $form = ['options' => ['#theme' => 'container']];
    $builder->buildTyped($form['options']);
    return [$form['options']['#title']];
}

function byValue(): array
{
    $form = ['options' => ['#theme' => 'container']];
    echo count($form['options']);
    return [$form['options']['#title']];
    //                       ^^^^^^^^ NonExistentArrayOffset: Array offset '#title' does not exist
}
