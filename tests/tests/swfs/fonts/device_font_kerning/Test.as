package {
import flash.display.*;
import flash.text.*;

// Two fonts share the same shape but differ in metric.
// TestFontEmbedded is the original rectangle font, embedded in the SWF
// under family EmbeddedFont. TestFontDevice has every glyph advance
// doubled. It is registered as device family DeviceFont in test.toml
// and installed system-wide so Flash Player finds it.
//
// Four TextFields render "abcd". One pair uses EmbeddedFont with
// embedFonts=true. The other uses DeviceFont with embedFonts=false,
// which sends the lookup through the device-font code path. Each pair
// is drawn twice, kerning off and kerning on.
//
// The trace prints all four widths. EmbeddedFont comes out at 40 both
// times. DeviceFont comes out at 80 both times. These numbers only
// line up if the device request really did go to DeviceFont and not
// to the embedded copy, and if classic TextField left the device case
// unkerned.
[SWF(width="100", height="100")]
public class Test extends Sprite {
    [Embed(source="TestFontEmbedded.ttf", fontName="EmbeddedFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var EmbeddedFontClass:Class;

    public function Test() {
        stage.scaleMode = "noScale";
        addTextField( 0, "EmbeddedFont", true,  false);
        addTextField(25, "EmbeddedFont", true,  true);
        addTextField(50, "DeviceFont",   false, false);
        addTextField(75, "DeviceFont",   false, true);
    }

    private function addTextField(y:Number, fontName:String, embed:Boolean, kerning:Boolean):void {
        var f:TextField = new TextField();
        f.type = "input";
        f.width = 100;
        f.height = 25;
        f.y = y;
        f.border = true;
        f.embedFonts = embed;
        var tf:TextFormat = new TextFormat(fontName, 10);
        tf.kerning = kerning;
        f.defaultTextFormat = tf;
        f.text = "abcd";
        trace(fontName + " kerning=" + kerning + " width: " + f.textWidth);
        addChild(f);
    }
}
}
