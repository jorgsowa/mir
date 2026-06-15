===description===
method call only valid after template narrowing
===file===
<?php
class Article {
    public function getTitle(): string {
        return "Article";
    }
}

class Photo {
    public function getThumbnail(): string {
        return "thumb.jpg";
    }
}

/**
 * @template TContent as Article|Photo
 * @param TContent $content
 */
function renderContent(Article|Photo $content): void {
    if ($content instanceof Article) {
        echo $content->getTitle();
    } elseif ($content instanceof Photo) {
        echo $content->getThumbnail();
    }
}

/**
 * Calling an undefined method before narrowing should error
 * @template TContent as Article|Photo
 * @param TContent $content
 */
function errorCase(Article|Photo $content): void {
    $content->undefinedMethod();
}
===expect===
RedundantCondition@21:14-21:39: Condition is always true/false for type 'bool'
UndefinedMethod@32:4-32:31: Method Article::undefinedMethod() does not exist
