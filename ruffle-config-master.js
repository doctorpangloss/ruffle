window.RufflePlayer = window.RufflePlayer || {};
window.RufflePlayer.config = {
  publicPath: "ruffle-master/",
  allowNetworking: "all",
  allowScriptAccess: true,
  autoplay: "on",
  splashScreen: true,
  unmuteOverlay: "hidden",
  logLevel: "warn",
  fontSources: [
    "fonts/LiberationSans-Regular.ttf",
    "fonts/LiberationSans-Bold.ttf",
    "fonts/LiberationSerif-Regular.ttf",
    "fonts/LiberationSerif-Bold.ttf",
    "fonts/LiberationMono-Regular.ttf",
    "fonts/LiberationMono-Bold.ttf",
  ],
  defaultFonts: {
    sans: ["Liberation Sans", "Arial", "Helvetica"],
    serif: ["Liberation Serif", "Times New Roman", "Times"],
    typewriter: ["Liberation Mono", "Courier New", "Courier"],
  },
  urlRewriteRules: [
    [
      /^https?:\/\/static01\.nyt\.com\/packages\/flash\/opinion\/20110620-morris-emulator\/Multics\.swf$/,
      "multics/Multics.swf",
    ],
    [
      /^https?:\/\/static01\.nyt\.com\/packages\/flash\/opinion\/20110620-morris-emulator\/(Left%20Comp_2|Left Comp_2)\.flv$/,
      "multics/Left%20Comp_2.flv",
    ],
    [
      /^https?:\/\/static01\.nyt\.com\/packages\/flash\/opinion\/20110620-morris-emulator\/(Right%20Comp_2|Right Comp_2)\.flv$/,
      "multics/Right%20Comp_2.flv",
    ],
    [
      /^https?:\/\/static01\.nyt\.com\/packages\/flash\/opinion\/20110620-morris-emulator\/textLayout_1\.0\.0\.595\.swz$/,
      "multics/textLayout_1.0.0.595.swz",
    ],
    [
      /^https?:\/\/fpdownload\.adobe\.com\/pub\/swz\/tlf\/1\.0\.0\.595\/textLayout_1\.0\.0\.595\.swz$/,
      "multics/textLayout_1.0.0.595.swz",
    ],
    [
      /^https?:\/\/new\.weedtowonder\.org\/ch2-domestication\.swf$/,
      "weed/ch2-domestication.swf",
    ],
    [
      /^https?:\/\/new\.weedtowonder\.org\/textLayout_2\.0\.0\.232\.swz$/,
      "weed/textLayout_2.0.0.232.swz",
    ],
    [
      /^https?:\/\/fpdownload\.adobe\.com\/pub\/swz\/tlf\/2\.0\.0\.232\/textLayout_2\.0\.0\.232\.swz$/,
      "weed/textLayout_2.0.0.232.swz",
    ],
    [/^.*textLayout_2\.0\.0\.232\.swz$/, "weed/textLayout_2.0.0.232.swz"],
    [/^.*textLayout_1\.0\.0\.595\.swz$/, "multics/textLayout_1.0.0.595.swz"],
    [
      /^https:\/\/swf-proxy\.appmana\.com\/proxy\/https?:\/\/web\.archive\.org\/web\/2id_\/(https?:\/\/.*)$/i,
      "https://swf-proxy.appmana.com/proxy/$1",
    ],
    [
      /^https?:\/\/(?!(?:doctorpangloss\.github\.io|swf-proxy\.appmana\.com|127\.0\.0\.1|localhost)(?::\d+)?\/)(.*)$/i,
      "https://swf-proxy.appmana.com/proxy/$&",
    ],
  ],
};
