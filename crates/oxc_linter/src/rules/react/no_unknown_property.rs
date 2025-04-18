use std::borrow::Cow;

use cow_utils::CowUtils;
use itertools::Itertools;
use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeName, JSXElementName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;
use serde::Deserialize;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    globals::is_valid_aria_property,
    rule::Rule,
    utils::get_jsx_attribute_name,
};

fn invalid_prop_on_tag(span: Span, prop: &str, tag: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Invalid property found")
        .with_help(format!("Property '{prop}' is only allowed on: {tag}"))
        .with_label(span)
}

fn data_lowercase_required(span: Span, suggested_prop: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "React does not recognize data-* props with uppercase characters on a DOM element",
    )
    .with_help(format!("Use '{suggested_prop}' instead"))
    .with_label(span)
}

fn unknown_prop_with_standard_name(span: Span, x1: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unknown property found")
        .with_help(format!("Use '{x1}' instead"))
        .with_label(span)
}

fn unknown_prop(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unknown property found")
        .with_help("Remove unknown property")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnknownProperty(Box<NoUnknownPropertyConfig>);

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoUnknownPropertyConfig {
    #[serde(default)]
    ignore: FxHashSet<Cow<'static, str>>,
    #[serde(default)]
    require_data_lowercase: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Disallow usage of unknown DOM property.
    ///
    /// ### Why is this bad?
    /// You can use unknown property name that has no effect.
    ///
    /// ### Example
    /// ```jsx
    ///  // Unknown properties
    ///  const Hello = <div class="hello">Hello World</div>;
    ///  const Alphabet = <div abc="something">Alphabet</div>;
    ///
    ///  // Invalid aria-* attribute
    ///  const IconButton = <div aria-foo="bar" />;
    /// ```
    NoUnknownProperty,
    react,
    restriction,
    pending
);

const ATTRIBUTE_TAGS_MAP_KEYS: [&str; 59] = [
    "abbr",
    "align",
    "allowFullScreen",
    "as",
    "autoPictureInPicture",
    "charset",
    "checked",
    "controls",
    "controlsList",
    // image is required for SVG support, all other tags are HTML.
    "crossOrigin",
    "disablePictureInPicture",
    "disableRemotePlayback",
    "displaystyle",
    // https://html.spec.whatwg.org/multipage/links.html#downloading-resources
    "download",
    "fill",
    "focusable",
    "imageSizes",
    "imageSrcSet",
    "loop",
    "mozAllowFullScreen",
    "muted",
    "noModule",
    // Media events allowed only on audio and video tags, see https://github.com/facebook/react/blob/256aefbea1449869620fb26f6ec695536ab453f5/CHANGELOG.md#notable-enhancements
    "onAbort",
    "onCanPlay",
    "onCanPlayThrough",
    "onCancel",
    "onClose",
    "onDurationChange",
    "onEmptied",
    "onEncrypted",
    "onEnded",
    "onError",
    "onLoad",
    "onLoadStart",
    "onLoadedData",
    "onLoadedMetadata",
    "onPause",
    "onPlay",
    "onPlaying",
    "onProgress",
    "onRateChange",
    "onResize",
    "onSeeked",
    "onSeeking",
    "onStalled",
    "onSuspend",
    "onTimeUpdate",
    "onVolumeChange",
    "onWaiting",
    "playsInline",
    "poster",
    "preload",
    "property",
    "returnValue",
    "scrolling",
    "valign",
    "viewBox",
    "webkitAllowFullScreen",
    "webkitDirectory",
];

const META: [&str; 1] = ["meta"];
const LINK: [&str; 1] = ["link"];
const VIDEO: [&str; 1] = ["video"];
const DIALOG: [&str; 1] = ["dialog"];
const AUDIO_VIDEO: [&str; 2] = ["audio", "video"];
const IFRAME_VIDEO: [&str; 2] = ["iframe", "video"];

const ATTRIBUTE_TAGS_MAP_VALUES: [&[&str]; 59] = [
    &["td", "th"],
    &[
        "applet", "caption", "col", "colgroup", "hr", "iframe", "img", "table", "tbody", "td",
        "tfoot", "th", "thead", "tr",
    ],
    &IFRAME_VIDEO,
    &["link"],
    &VIDEO,
    &META,
    &["input"],
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &["audio", "image", "img", "link", "script", "video"],
    &VIDEO,
    &AUDIO_VIDEO,
    &["math"],
    &["a", "area"],
    &[
        "altGlyph",
        "animate",
        "animateColor",
        "animateMotion",
        "animateTransform",
        "circle",
        "ellipse",
        "g",
        "line",
        "marker",
        "mask",
        "path",
        "polygon",
        "polyline",
        "rect",
        "set",
        "svg",
        "symbol",
        "text",
        "textPath",
        "tref",
        "tspan",
        "use",
    ],
    &["svg"],
    &LINK,
    &LINK,
    &AUDIO_VIDEO,
    &IFRAME_VIDEO,
    &AUDIO_VIDEO,
    &["script"],
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &DIALOG,
    &DIALOG,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &["audio", "iframe", "img", "link", "picture", "script", "source", "video"],
    &["iframe", "img", "link", "object", "picture", "script", "source"],
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &AUDIO_VIDEO,
    &VIDEO,
    &VIDEO,
    &AUDIO_VIDEO,
    &META,
    &DIALOG,
    &["iframe"],
    &["col", "colgroup", "tbody", "td", "tfoot", "th", "thead", "tr"],
    &["marker", "pattern", "svg", "symbol", "view"],
    &IFRAME_VIDEO,
    &["input"],
];

const DOM_PROPERTIES_NAMES: [&str; 560] = [
    "accentHeight",
    "accept",
    "acceptCharset",
    "accessKey",
    "accumulate",
    "action",
    "additive",
    "alignmentBaseline",
    "allow",
    "alphabetic",
    "alt",
    "amplitude",
    "arabicForm",
    "as",
    "ascent",
    "async",
    "attributeName",
    "attributeType",
    "autoCapitalize",
    "autoComplete",
    "autoCorrect",
    "autoFocus",
    "autoPictureInPicture",
    "autoPlay",
    "autoSave",
    "azimuth",
    "baseFrequency",
    "baseProfile",
    "baselineShift",
    "bbox",
    "begin",
    "bias",
    "border",
    "buffered",
    "by",
    "calcMode",
    "capHeight",
    "capture",
    "cellPadding",
    "cellSpacing",
    "challenge",
    "children",
    "cite",
    "classID",
    "className",
    "clip",
    "clipPath",
    "clipPathUnits",
    "clipRule",
    "code",
    "codeBase",
    "colSpan",
    "color",
    "colorInterpolation",
    "colorInterpolationFilters",
    "colorProfile",
    "colorRendering",
    "cols",
    "content",
    "contentEditable",
    "contentScriptType",
    "contentStyleType",
    "contextMenu",
    "controls",
    "controlsList",
    "coords",
    "crossOrigin",
    "csp",
    "cursor",
    "cx",
    "cy",
    "d",
    "dangerouslySetInnerHTML",
    "data",
    "dateTime",
    "decelerate",
    "decoding",
    "default",
    "defaultChecked",
    "defaultValue",
    "defer",
    "descent",
    "diffuseConstant",
    "dir",
    "direction",
    "disablePictureInPicture",
    "disableRemotePlayback",
    "disabled",
    "display",
    "divisor",
    "dominantBaseline",
    "draggable",
    "dur",
    "dx",
    "dy",
    "edgeMode",
    "elevation",
    "enableBackground",
    "encType",
    "end",
    "enterKeyHint",
    "exponent",
    "exportParts",
    "fill",
    "fillOpacity",
    "fillRule",
    "filter",
    "filterRes",
    "filterUnits",
    "floodColor",
    "floodOpacity",
    "fontFamily",
    "fontSize",
    "fontSizeAdjust",
    "fontStretch",
    "fontStyle",
    "fontVariant",
    "fontWeight",
    "form",
    "formAction",
    "formEncType",
    "formMethod",
    "formNoValidate",
    "formTarget",
    "format",
    "fr",
    "frameBorder",
    "from",
    "fx",
    "fy",
    "g1",
    "g2",
    "glyphName",
    "glyphOrientationHorizontal",
    "glyphOrientationVertical",
    "glyphRef",
    "gradientTransform",
    "gradientUnits",
    "hanging",
    "headers",
    "height",
    "hidden",
    "high",
    "horizAdvX",
    "horizOriginX",
    "href",
    "hrefLang",
    "hreflang",
    "htmlFor",
    "httpEquiv",
    "icon",
    "id",
    "ideographic",
    "imageRendering",
    "imageSizes",
    "imageSrcSet",
    "importance",
    "in",
    "in2",
    "inert",
    "inputMode",
    "integrity",
    "intercept",
    "isMap",
    "itemID",
    "itemProp",
    "itemRef",
    "itemScope",
    "itemType",
    "k",
    "k1",
    "k2",
    "k3",
    "k4",
    "kernelMatrix",
    "kernelUnitLength",
    "kerning",
    "key",
    "keyParams",
    "keyPoints",
    "keySplines",
    "keyTimes",
    "keyType",
    "kind",
    "label",
    "lang",
    "language",
    "lengthAdjust",
    "letterSpacing",
    "lightingColor",
    "limitingConeAngle",
    "list",
    "loading",
    "local",
    "loop",
    "low",
    "manifest",
    "marginHeight",
    "marginWidth",
    "markerEnd",
    "markerHeight",
    "markerMid",
    "markerStart",
    "markerUnits",
    "markerWidth",
    "mask",
    "maskContentUnits",
    "maskUnits",
    "mathematical",
    "max",
    "maxLength",
    "media",
    "mediaGroup",
    "method",
    "min",
    "minLength",
    "mode",
    "multiple",
    "muted",
    "name",
    "noValidate",
    "nonce",
    "numOctaves",
    "offset",
    "onAbort",
    "onAbortCapture",
    "onAnimationEnd",
    "onAnimationEndCapture",
    "onAnimationIteration",
    "onAnimationStart",
    "onAnimationStartCapture",
    "onAuxClick",
    "onAuxClickCapture",
    "onBeforeInput",
    "onBeforeInputCapture",
    "onBlur",
    "onBlurCapture",
    "onCanPlay",
    "onCanPlayCapture",
    "onCanPlayThrough",
    "onCanPlayThroughCapture",
    "onChange",
    "onChangeCapture",
    "onClick",
    "onClickCapture",
    "onCompositionEnd",
    "onCompositionEndCapture",
    "onCompositionStart",
    "onCompositionStartCapture",
    "onCompositionUpdate",
    "onCompositionUpdateCapture",
    "onContextMenu",
    "onContextMenuCapture",
    "onCopy",
    "onCopyCapture",
    "onCut",
    "onCutCapture",
    "onDoubleClick",
    "onDoubleClickCapture",
    "onDrag",
    "onDragCapture",
    "onDragEnd",
    "onDragEndCapture",
    "onDragEnter",
    "onDragEnterCapture",
    "onDragExit",
    "onDragExitCapture",
    "onDragLeave",
    "onDragLeaveCapture",
    "onDragOver",
    "onDragOverCapture",
    "onDragStart",
    "onDragStartCapture",
    "onDrop",
    "onDropCapture",
    "onDurationChange",
    "onDurationChangeCapture",
    "onEmptied",
    "onEmptiedCapture",
    "onEncrypted",
    "onEncryptedCapture",
    "onEnded",
    "onEndedCapture",
    "onError",
    "onErrorCapture",
    "onFocus",
    "onFocusCapture",
    "onGotPointerCaptureCapture",
    "onInput",
    "onInputCapture",
    "onInvalid",
    "onInvalidCapture",
    "onKeyDown",
    "onKeyDownCapture",
    "onKeyPress",
    "onKeyPressCapture",
    "onKeyUp",
    "onKeyUpCapture",
    "onLoad",
    "onLoadCapture",
    "onLoadStart",
    "onLoadStartCapture",
    "onLoadedData",
    "onLoadedDataCapture",
    "onLoadedMetadata",
    "onLoadedMetadataCapture",
    "onLostPointerCapture",
    "onLostPointerCaptureCapture",
    "onMouseDown",
    "onMouseDownCapture",
    "onMouseEnter",
    "onMouseLeave",
    "onMouseMove",
    "onMouseMoveCapture",
    "onMouseOut",
    "onMouseOutCapture",
    "onMouseOver",
    "onMouseOverCapture",
    "onMouseUp",
    "onMouseUpCapture",
    "onPaste",
    "onPasteCapture",
    "onPause",
    "onPauseCapture",
    "onPlay",
    "onPlayCapture",
    "onPlaying",
    "onPlayingCapture",
    "onPointerCancel",
    "onPointerCancelCapture",
    "onPointerDown",
    "onPointerDownCapture",
    "onPointerEnter",
    "onPointerEnterCapture",
    "onPointerLeave",
    "onPointerLeaveCapture",
    "onPointerMove",
    "onPointerMoveCapture",
    "onPointerOut",
    "onPointerOutCapture",
    "onPointerOver",
    "onPointerOverCapture",
    "onPointerUp",
    "onPointerUpCapture",
    "onProgress",
    "onProgressCapture",
    "onRateChange",
    "onRateChangeCapture",
    "onReset",
    "onResetCapture",
    "onResize",
    "onScroll",
    "onScrollCapture",
    "onSeeked",
    "onSeekedCapture",
    "onSeeking",
    "onSeekingCapture",
    "onSelect",
    "onSelectCapture",
    "onStalled",
    "onStalledCapture",
    "onSubmit",
    "onSubmitCapture",
    "onSuspend",
    "onSuspendCapture",
    "onTimeUpdate",
    "onTimeUpdateCapture",
    "onToggle",
    "onTouchCancel",
    "onTouchCancelCapture",
    "onTouchEnd",
    "onTouchEndCapture",
    "onTouchMove",
    "onTouchMoveCapture",
    "onTouchStart",
    "onTouchStartCapture",
    "onTransitionEnd",
    "onTransitionEndCapture",
    "onVolumeChange",
    "onVolumeChangeCapture",
    "onWaiting",
    "onWaitingCapture",
    "onWheel",
    "onWheelCapture",
    "opacity",
    "open",
    "operator",
    "optimum",
    "order",
    "orient",
    "orientation",
    "origin",
    "overflow",
    "overlinePosition",
    "overlineThickness",
    "paintOrder",
    "panose1",
    "part",
    "path",
    "pathLength",
    "pattern",
    "patternContentUnits",
    "patternTransform",
    "patternUnits",
    "ping",
    "placeholder",
    "pointerEvents",
    "points",
    "pointsAtX",
    "pointsAtY",
    "pointsAtZ",
    "poster",
    "preload",
    "preserveAlpha",
    "preserveAspectRatio",
    "primitiveUnits",
    "profile",
    "property",
    "r",
    "radioGroup",
    "radius",
    "readOnly",
    "ref",
    "refX",
    "refY",
    "referrerPolicy",
    "rel",
    "rendering-intent",
    "repeatCount",
    "repeatDur",
    "required",
    "requiredExtensions",
    "requiredFeatures",
    "restart",
    "result",
    "results",
    "reversed",
    "role",
    "rotate",
    "rowSpan",
    "rows",
    "rx",
    "ry",
    "sandbox",
    "scale",
    "scope",
    "seamless",
    "security",
    "seed",
    "selected",
    "shape",
    "shapeRendering",
    "size",
    "sizes",
    "slope",
    "slot",
    "spacing",
    "span",
    "specularConstant",
    "specularExponent",
    "speed",
    "spellCheck",
    "spreadMethod",
    "src",
    "srcDoc",
    "srcLang",
    "srcSet",
    "start",
    "startOffset",
    "stdDeviation",
    "stemh",
    "stemv",
    "step",
    "stitchTiles",
    "stopColor",
    "stopOpacity",
    "strikethroughPosition",
    "strikethroughThickness",
    "string",
    "stroke",
    "strokeDasharray",
    "strokeDashoffset",
    "strokeLinecap",
    "strokeLinejoin",
    "strokeMiterlimit",
    "strokeOpacity",
    "strokeWidth",
    "style",
    "summary",
    "suppressContentEditableWarning",
    "suppressHydrationWarning",
    "surfaceScale",
    "systemLanguage",
    "tabIndex",
    "tableValues",
    "target",
    "targetX",
    "targetY",
    "textAnchor",
    "textDecoration",
    "textLength",
    "textRendering",
    "title",
    "to",
    "transform",
    "transformOrigin",
    "translate",
    "type",
    "u1",
    "u2",
    "underlinePosition",
    "underlineThickness",
    "unicode",
    "unicodeBidi",
    "unicodeRange",
    "unitsPerEm",
    "useMap",
    "vAlphabetic",
    "vHanging",
    "vIdeographic",
    "vMathematical",
    "value",
    "values",
    "vectorEffect",
    "version",
    "vertAdvY",
    "vertOriginX",
    "vertOriginY",
    "viewBox",
    "viewTarget",
    "visibility",
    "width",
    "widths",
    "wmode",
    "wordSpacing",
    "wrap",
    "writingMode",
    "x",
    "x1",
    "x2",
    "xChannelSelector",
    "xHeight",
    "xlinkActuate",
    "xlinkArcrole",
    "xlinkHref",
    "xlinkRole",
    "xlinkShow",
    "xlinkTitle",
    "xlinkType",
    "xmlBase",
    "xmlLang",
    "xmlSpace",
    "xmlns",
    "xmlnsXlink",
    "y",
    "y1",
    "y2",
    "yChannelSelector",
    "z",
    "zoomAndPan",
];

const DOM_ATTRIBUTES_TO_CAMEL_KEYS: [&str; 88] = [
    "accent-height",
    "accept-charset",
    "alignment-baseline",
    "arabic-form",
    "baseline-shift",
    "cap-height",
    "class",
    "clip-path",
    "clip-rule",
    "color-interpolation",
    "color-interpolation-filters",
    "color-profile",
    "color-rendering",
    "crossorigin",
    "dominant-baseline",
    "enable-background",
    "fill-opacity",
    "fill-rule",
    "flood-color",
    "flood-opacity",
    "font-family",
    "font-size",
    "font-size-adjust",
    "font-stretch",
    "font-style",
    "font-variant",
    "font-weight",
    "for",
    "glyph-name",
    "glyph-orientation-horizontal",
    "glyph-orientation-vertical",
    "horiz-adv-x",
    "horiz-origin-x",
    "http-equiv",
    "image-rendering",
    "letter-spacing",
    "lighting-color",
    "marker-end",
    "marker-mid",
    "marker-start",
    "nomodule",
    "overline-position",
    "overline-thickness",
    "paint-order",
    "panose-1",
    "pointer-events",
    "rendering-intent",
    "shape-rendering",
    "stop-color",
    "stop-opacity",
    "strikethrough-position",
    "strikethrough-thickness",
    "stroke-dasharray",
    "stroke-dashoffset",
    "stroke-linecap",
    "stroke-linejoin",
    "stroke-miterlimit",
    "stroke-opacity",
    "stroke-width",
    "text-anchor",
    "text-decoration",
    "text-rendering",
    "underline-position",
    "underline-thickness",
    "unicode-bidi",
    "unicode-range",
    "units-per-em",
    "v-alphabetic",
    "v-hanging",
    "v-ideographic",
    "v-mathematical",
    "vector-effect",
    "vert-adv-y",
    "vert-origin-x",
    "vert-origin-y",
    "word-spacing",
    "writing-mode",
    "x-height",
    "xlink:actuate",
    "xlink:arcrole",
    "xlink:href",
    "xlink:role",
    "xlink:show",
    "xlink:title",
    "xlink:type",
    "xml:base",
    "xml:lang",
    "xml:space",
];

const DOM_ATTRIBUTES_TO_CAMEL_VALUES: [&str; 88] = [
    "accentHeight",
    "acceptCharset",
    "alignmentBaseline",
    "arabicForm",
    "baselineShift",
    "capHeight",
    "className",
    "clipPath",
    "clipRule",
    "colorInterpolation",
    "colorInterpolationFilters",
    "colorProfile",
    "colorRendering",
    "crossOrigin",
    "dominantBaseline",
    "enableBackground",
    "fillOpacity",
    "fillRule",
    "floodColor",
    "floodOpacity",
    "fontFamily",
    "fontSize",
    "fontSizeAdjust",
    "fontStretch",
    "fontStyle",
    "fontVariant",
    "fontWeight",
    "htmlFor",
    "glyphName",
    "glyphOrientationHorizontal",
    "glyphOrientationVertical",
    "horizAdvX",
    "horizOriginX",
    "httpEquiv",
    "imageRendering",
    "letterSpacing",
    "lightingColor",
    "markerEnd",
    "markerMid",
    "markerStart",
    "noModule",
    "overlinePosition",
    "overlineThickness",
    "paintOrder",
    "panose1",
    "pointerEvents",
    "renderingIntent",
    "shapeRendering",
    "stopColor",
    "stopOpacity",
    "strikethroughPosition",
    "strikethroughThickness",
    "strokeDasharray",
    "strokeDashoffset",
    "strokeLinecap",
    "strokeLinejoin",
    "strokeMiterlimit",
    "strokeOpacity",
    "strokeWidth",
    "textAnchor",
    "textDecoration",
    "textRendering",
    "underlinePosition",
    "underlineThickness",
    "unicodeBidi",
    "unicodeRange",
    "unitsPerEm",
    "vAlphabetic",
    "vHanging",
    "vIdeographic",
    "vMathematical",
    "vectorEffect",
    "vertAdvY",
    "vertOriginX",
    "vertOriginY",
    "wordSpacing",
    "writingMode",
    "xHeight",
    "xlinkActuate",
    "xlinkArcrole",
    "xlinkHref",
    "xlinkRole",
    "xlinkShow",
    "xlinkTitle",
    "xlinkType",
    "xmlBase",
    "xmlLang",
    "xmlSpace",
];

const DOM_PROPERTIES_IGNORE_CASE: [&str; 5] = [
    "charset",
    "allowFullScreen",
    "webkitAllowFullScreen",
    "mozAllowFullScreen",
    "webkitDirectory",
];

/// Checks if an attribute name is a valid `data-*` attribute:
/// - Name starts with "data-" and has alphanumeric words (browsers require lowercase, but React and TS lowercase them),
/// - Does not start with any casing of "xml"
/// - Words are separated by hyphens (-) (which is also called "kebab case" or "dash case")
fn is_valid_data_attr(name: &str) -> bool {
    if !name.starts_with("data-") {
        return false;
    }

    if name.cow_to_ascii_lowercase().starts_with("data-xml") {
        return false;
    }

    let data_name = &name["data-".len()..];
    if data_name.is_empty() {
        return false;
    }

    data_name.chars().all(|c| c != ':')
}

/// Checks if a tag name matches the HTML tag conventions.
fn matches_html_tag_conventions(tag: &str) -> bool {
    tag.char_indices().all(|(i, c)| if i == 0 { c.is_ascii_lowercase() } else { c != '-' })
}

fn normalize_attribute_case(name: &str) -> &str {
    DOM_PROPERTIES_IGNORE_CASE
        .iter()
        .find(|camel_name| camel_name.eq_ignore_ascii_case(name))
        .unwrap_or(&name)
}
fn has_uppercase(name: &str) -> bool {
    name.contains(char::is_uppercase)
}

impl Rule for NoUnknownProperty {
    fn from_configuration(value: serde_json::Value) -> Self {
        value
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .map_or_else(Self::default, |value| Self(Box::new(value)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(el) = &node.kind() else {
            return;
        };
        let JSXElementName::Identifier(ident) = &el.name else {
            return;
        };
        let el_type = ident.name.as_str();

        // fbt/fbs nodes are bonkers, let's not go there
        if !el_type.starts_with(char::is_lowercase) || el_type == "fbt" || el_type == "fbs" {
            return;
        }

        let is_valid_html_tag = matches_html_tag_conventions(el_type)
            && el.attributes.iter().all(|attr| {
                let JSXAttributeItem::Attribute(jsx_attr) = attr else {
                    return true;
                };
                let JSXAttributeName::Identifier(ident) = &jsx_attr.name else {
                    return true;
                };
                ident.name.as_str() != "is"
            });

        el.attributes
            .iter()
            .filter_map(|attr| match &attr {
                JSXAttributeItem::Attribute(regular) => Some(&**regular),
                JSXAttributeItem::SpreadAttribute(_) => None,
            })
            .for_each(|attr| {
                let span = attr.name.span();
                let actual_name = get_jsx_attribute_name(&attr.name);
                if self.0.ignore.contains(&(actual_name)) {
                    return;
                }
                if is_valid_data_attr(&actual_name) {
                    if self.0.require_data_lowercase && has_uppercase(&actual_name) {
                        ctx.diagnostic(data_lowercase_required(
                            span,
                            &actual_name.cow_to_ascii_lowercase(),
                        ));
                    }
                    return;
                }
                if is_valid_aria_property(&actual_name) || !is_valid_html_tag {
                    return;
                }
                let name = normalize_attribute_case(&actual_name);
                if let Ok(index) = ATTRIBUTE_TAGS_MAP_KEYS.binary_search(&name) {
                    let tags = ATTRIBUTE_TAGS_MAP_VALUES[index];
                    if tags.binary_search(&el_type).is_err() {
                        ctx.diagnostic(invalid_prop_on_tag(
                            span,
                            &actual_name,
                            &tags.iter().join(", "),
                        ));
                    }
                    return;
                }

                if DOM_PROPERTIES_NAMES.binary_search(&name).is_ok() {
                    return;
                }

                let right = &name;
                let result = DOM_PROPERTIES_NAMES.binary_search_by(|left| {
                    let l = std::cmp::min(left.len(), right.len());

                    let lhs = &left.as_bytes()[..l];
                    let rhs = &right.as_bytes()[..l];

                    for i in 0..l {
                        match lhs[i].to_ascii_lowercase().cmp(&rhs[i].to_ascii_lowercase()) {
                            std::cmp::Ordering::Equal => (),
                            non_eq => return non_eq,
                        }
                    }

                    left.len().cmp(&right.len())
                });

                let prop = result.map(|i| DOM_PROPERTIES_NAMES[i]).or_else(|_| {
                    DOM_ATTRIBUTES_TO_CAMEL_KEYS
                        .binary_search(&name)
                        .map(|index| DOM_ATTRIBUTES_TO_CAMEL_VALUES[index])
                });

                if let Ok(prop) = prop {
                    ctx.diagnostic(unknown_prop_with_standard_name(span, prop));
                } else {
                    ctx.diagnostic(unknown_prop(span));
                }
            });
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"<App class="bar" />;"#, None),
        (r#"<App for="bar" />;"#, None),
        (r#"<App someProp="bar" />;"#, None),
        (r#"<Foo.bar for="bar" />;"#, None),
        (r#"<App accept-charset="bar" />;"#, None),
        (r#"<App http-equiv="bar" />;"#, None),
        (r#"<App xlink:href="bar" />;"#, None),
        (r#"<App clip-path="bar" />;"#, None),
        (r#"<div className="bar"></div>;"#, None),
        (r"<div onMouseDown={this._onMouseDown}></div>;", None),
        (r#"<a href="someLink" download="foo">Read more</a>"#, None),
        (r#"<area download="foo" />"#, None),
        (r#"<img src="cat_keyboard.jpeg" alt="A cat sleeping on a keyboard" align="top" />"#, None),
        (r#"<input type="password" required />"#, None),
        (r#"<input ref={this.input} type="radio" />"#, None),
        (r#"<input type="file" webkitdirectory="" />"#, None),
        (r#"<input type="file" webkitDirectory="" />"#, None),
        (r#"<div inert children="anything" />"#, None),
        (r#"<iframe scrolling="?" onLoad={a} onError={b} align="top" />"#, None),
        (r#"<input key="bar" type="radio" />"#, None),
        (r"<button disabled>You cannot click me</button>;", None),
        (
            r#"<svg key="lock" viewBox="box" fill={10} d="d" stroke={1} strokeWidth={2} strokeLinecap={3} strokeLinejoin={4} transform="something" clipRule="else" x1={5} x2="6" y1="7" y2="8"></svg>"#,
            None,
        ),
        (r#"<g fill="\#7B82A0" fillRule="evenodd"></g>"#, None),
        (r#"<mask fill="\#7B82A0"></mask>"#, None),
        (r#"<symbol fill="\#7B82A0"></symbol>"#, None),
        (r#"<meta property="og:type" content="website" />"#, None),
        (
            r#"<input type="checkbox" checked={checked} disabled={disabled} id={id} onChange={onChange} />"#,
            None,
        ),
        (r"<video playsInline />", None),
        (r"<img onError={foo} onLoad={bar} />", None),
        (r"<picture inert={false} onError={foo} onLoad={bar} />", None),
        (r"<iframe onError={foo} onLoad={bar} />", None),
        (r"<script onLoad={bar} onError={foo} />", None),
        (r"<source onLoad={bar} onError={foo} />", None),
        (r"<link onLoad={bar} onError={foo} />", None),
        (
            r#"<link rel="preload" as="image" href="someHref" imageSrcSet="someImageSrcSet" imageSizes="someImageSizes" />"#,
            None,
        ),
        (r"<object onLoad={bar} />", None),
        (r"<video allowFullScreen webkitAllowFullScreen mozAllowFullScreen />", None),
        (r"<iframe allowFullScreen webkitAllowFullScreen mozAllowFullScreen />", None),
        (r#"<table border="1" />"#, None),
        (r#"<th abbr="abbr" />"#, None),
        (r#"<td abbr="abbr" />"#, None),
        (r"<div onPointerDown={this.onDown} onPointerUp={this.onUp} />", None),
        (r#"<input type="checkbox" defaultChecked={this.state.checkbox} />"#, None),
        (
            r"<div onTouchStart={this.startAnimation} onTouchEnd={this.stopAnimation} onTouchCancel={this.cancel} onTouchMove={this.move} onMouseMoveCapture={this.capture} onTouchCancelCapture={this.log} />",
            None,
        ),
        (r#"<meta charset="utf-8" />;"#, None),
        (r#"<meta charSet="utf-8" />;"#, None),
        (r#"<div class="foo" is="my-elem"></div>;"#, None),
        (r#"<div {...this.props} class="foo" is="my-elem"></div>;"#, None),
        (r#"<atom-panel class="foo"></atom-panel>;"#, None),
        (r#"<div data-foo="bar"></div>;"#, None),
        (r#"<div data-foo-bar="baz"></div>;"#, None),
        (r#"<div data-parent="parent"></div>;"#, None),
        (r#"<div data-index-number="1234"></div>;"#, None),
        (r#"<div data-e2e-id="5678"></div>;"#, None),
        (r#"<div data-testID="bar" data-under_sCoRe="bar" />;"#, None),
        (
            r#"<div data-testID="bar" data-under_sCoRe="bar" />;"#,
            Some(serde_json::json!([{ "requireDataLowercase": false }])),
        ),
        (r#"<div class="bar"></div>;"#, Some(serde_json::json!([{ "ignore": ["class"] }]))),
        (r#"<div someProp="bar"></div>;"#, Some(serde_json::json!([{ "ignore": ["someProp"] }]))),
        (r"<div css={{flex: 1}}></div>;", Some(serde_json::json!([{ "ignore": ["css"] }]))),
        (r#"<button aria-haspopup="true">Click me to open pop up</button>;"#, None),
        (r#"<button aria-label="Close" onClick={someThing.close} />;"#, None),
        (r"<script crossOrigin noModule />", None),
        (r"<audio crossOrigin />", None),
        (r"<svg focusable><image crossOrigin /></svg>", None),
        (r"<details onToggle={this.onToggle}>Some details</details>", None),
        (
            r#"<path fill="pink" d="M 10,30 A 20,20 0,0,1 50,30 A 20,20 0,0,1 90,30 Q 90,60 50,90 Q 10,60 10,30 z"></path>"#,
            None,
        ),
        (r#"<line fill="pink" x1="0" y1="80" x2="100" y2="20"></line>"#, None),
        (r#"<link as="audio">Audio content</link>"#, None),
        (
            r#"<video controlsList="nodownload" controls={this.controls} loop={true} muted={false} src={this.videoSrc} playsInline={true} onResize={this.onResize}></video>"#,
            None,
        ),
        (
            r#"<audio controlsList="nodownload" controls={this.controls} crossOrigin="anonymous" disableRemotePlayback loop muted preload="none" src="something" onAbort={this.abort} onDurationChange={this.durationChange} onEmptied={this.emptied} onEnded={this.end} onError={this.error} onResize={this.onResize}></audio>"#,
            None,
        ),
        (
            r#"<marker id={markerId} viewBox="0 0 2 2" refX="1" refY="1" markerWidth="1" markerHeight="1" orient="auto" />"#,
            None,
        ),
        (r#"<pattern id="pattern" viewBox="0,0,10,10" width="10%" height="10%" />"#, None),
        (r#"<symbol id="myDot" width="10" height="10" viewBox="0 0 2 2" />"#, None),
        (r#"<view id="one" viewBox="0 0 100 100" />"#, None),
        (r#"<hr align="top" />"#, None),
        (r#"<applet align="top" />"#, None),
        (r#"<marker fill="\#000" />"#, None),
        (
            r#"<dialog onClose={handler} open id="dialog" returnValue="something" onCancel={handler2} />"#,
            None,
        ),
        (
            r#"
			        <table align="top">
			          <caption align="top">Table Caption</caption>
			          <colgroup valign="top" align="top">
			            <col valign="top" align="top"/>
			          </colgroup>
			          <thead valign="top" align="top">
			            <tr valign="top" align="top">
			              <th valign="top" align="top">Header</th>
			              <td valign="top" align="top">Cell</td>
			            </tr>
			          </thead>
			          <tbody valign="top" align="top" />
			          <tfoot valign="top" align="top" />
			        </table>
			      "#,
            None,
        ),
        (r#"<fbt desc="foo" doNotExtract />;"#, None),
        (r#"<fbs desc="foo" doNotExtract />;"#, None),
        (r#"<math displaystyle="true" />;"#, None),
        (
            r#"
			        <div className="App" data-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash="customValue">
			          Hello, world!
			        </div>
			      "#,
            None,
        ),
    ];

    let fail = vec![
        (r#"<div allowTransparency="true" />"#, None),
        (r#"<div hasOwnProperty="should not be allowed property"></div>;"#, None),
        (r#"<div abc="should not be allowed property"></div>;"#, None),
        (r#"<div aria-fake="should not be allowed property"></div>;"#, None),
        (r#"<div someProp="bar"></div>;"#, None),
        (r#"<div class="bar"></div>;"#, None),
        (r#"<div for="bar"></div>;"#, None),
        (r#"<div accept-charset="bar"></div>;"#, None),
        (r#"<div http-equiv="bar"></div>;"#, None),
        (r#"<div accesskey="bar"></div>;"#, None),
        (r#"<div onclick="bar"></div>;"#, None),
        (r#"<div onmousedown="bar"></div>;"#, None),
        (r#"<div onMousedown="bar"></div>;"#, None),
        (r#"<use xlink:href="bar" />;"#, None),
        (r#"<rect clip-path="bar" />;"#, None),
        (r"<script crossorigin nomodule />", None),
        (r"<div crossorigin />", None),
        (r"<div crossOrigin />", None),
        (r#"<div as="audio" />"#, None),
        (
            r"<div onAbort={this.abort} onDurationChange={this.durationChange} onEmptied={this.emptied} onEnded={this.end} onResize={this.resize} onError={this.error} />",
            None,
        ),
        (r"<div onLoad={this.load} />", None),
        (r#"<div fill="pink" />"#, None),
        (
            r"<div controls={this.controls} loop={true} muted={false} src={this.videoSrc} playsInline={true} allowFullScreen></div>",
            None,
        ),
        (r#"<div download="foo" />"#, None),
        (r#"<div imageSrcSet="someImageSrcSet" />"#, None),
        (r#"<div imageSizes="someImageSizes" />"#, None),
        (r#"<div data-xml-anything="invalid" />"#, None),
        (
            r#"<div data-testID="bar" data-under_sCoRe="bar" />;"#,
            Some(serde_json::json!([{ "requireDataLowercase": true }])),
        ),
        (r#"<div abbr="abbr" />"#, None),
        (r#"<div webkitDirectory="" />"#, None),
        (r#"<div webkitdirectory="" />"#, None),
        (
            r#"
			        <div className="App" data-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash:c="customValue">
			          Hello, world!
			        </div>
			      "#,
            None,
        ),
    ];

    Tester::new(NoUnknownProperty::NAME, NoUnknownProperty::PLUGIN, pass, fail).test_and_snapshot();
}
