# Otaniemiemen lukion HSL-infotaulu
> [!NOTE]
> Heitetty kasaan päivässä, eli linjojen muuttaminen vaati lähdekoodin tökkimistä.

![](/img/image.png)

## Oman infotaulun pystyttäminen
> [!NOTE]
> Ohje on tarkoituksella tehty mahdollisimman yksinkertaiseksi ja ymmärrettäväksi.

1. Rekistöröidy ja hae kehittäjäavain (api-key) osoitteesta `https://digitransit.fi/`

2. Luo .env kansion palvelimelle sisällöllä:
    ```env
    API_KEY="<hsl-api-key>"
    STATIC_DIR="/frontend/static"
    ```

3. Jos olet tehnyt lähdekoodin muutoksia, rakenna kontti uudelleen komennolla `docker build . -t hsl`

4. Käynnistä palvelin komennolla `docker run --rm -it --mount -p 3060:3060 type=bind,src="<polku .env kansioon>",target=/.env hsl`

5. Infotaulu on nyt näkyvissä lokaalisti osoitteessa `localhost:3060`

