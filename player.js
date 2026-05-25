const params = new URLSearchParams(window.location.search);
const url = params.get("url");
const movie = document.getElementById("movie");

if (url) {
  const ruffle = window.RufflePlayer.newest();
  const player = ruffle.createPlayer();
  movie.append(player);
  player.ruffle().load(url);
}
