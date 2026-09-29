<p align="center">
  <a href="./README.md">English</a> · <strong>Português (Brasil)</strong>
</p>

<h1 align="center">Mado</h1>

<p align="center">
  <strong>Um cliente desktop nativo para Linux do ChatGPT Web.</strong><br>
  Rust · Relm4 · GTK 4 · WebKitGTK 6
</p>

> [!IMPORTANT]
> O Mado está em desenvolvimento ativo. Ainda não existe uma versão estável empacotada; por enquanto, o uso é a partir do código-fonte.

## O que é o Mado?

O Mado mantém a experiência real do ChatGPT Web dentro de um aplicativo pequeno e nativo para Linux, em vez de reconstruir o ChatGPT como cliente de API ou distribuir um runtime Chromium próprio.

- **Rust + Relm4**
- **GTK 4**
- **WebKitGTK 6**
- **Wayland em primeiro lugar**, mantendo compatibilidade com X11/XWayland quando for razoável.
- **Armazenamento seguindo XDG** para estado do aplicativo e dos perfis.

O Mado **não** usa Electron, Tauri, Qt WebEngine nem um Chromium empacotado.

## Estado atual

| Área | Estado |
| --- | --- |
| Aplicativo nativo GTK/Relm4 | Funcionando |
| Login + sessão WebKit persistente | Funcionando |
| Perfis persistentes e isolados | Motor pronto; UI planejada |
| Navegação confiável + navegador externo | Funcionando |
| Uploads + downloads normais/gerados nativos | Funcionando |
| Instância única + estado da janela | Funcionando |
| Recuperação limitada de falhas do WebKit | Funcionando |
| Recuperação local de rascunhos não enviados | Funcionando |
| Notificações de conclusão | Planejado em [#17](https://github.com/millesant/mado/issues/17) |
| UI de perfis/configurações | Planejado em [#18](https://github.com/millesant/mado/issues/18) |
| Diagnósticos sanitizados/privacidade | Planejado em [#19](https://github.com/millesant/mado/issues/19) |
| ChatGPT Voice completo no WebKitGTK 2.54 | Limitação do runtime; [#26](https://github.com/millesant/mado/issues/26) |

Veja [Compatibilidade WebKitGTK](./docs/WEBKIT_COMPAT.md) para os detalhes dos testes.

## Compilar e executar

A base de desenvolvimento validada é Fedora Linux 44.

    sudo dnf install rust cargo pkgconf-pkg-config gtk4-devel webkitgtk6.0-devel

    git clone https://github.com/millesant/mado.git
    cd mado
    cargo run

Mais detalhes: [docs/BUILDING.md](./docs/BUILDING.md).

## Privacidade e segurança

O Mado trata a sessão autenticada do WebKit como uma fronteira de segurança.

- Cookies de autenticação e estado sensível permanecem sob controle do WebKit sempre que possível.
- Perfis usam armazenamento WebKit separado em diretórios XDG.
- Links de terceiros seguem uma política explícita de navegação nativa.
- Mensagens JavaScript/nativo são tipadas, versionadas, isoladas e limitadas.
- Downloads usam nomes sanitizados, arquivos parciais e finalização sem sobrescrever arquivos existentes.
- Cookies, tokens, conteúdo de prompts e HTML bruto ficam fora do design de diagnósticos.

A recuperação local de rascunhos mantém uma cópia adicional do texto não enviado no estado XDG do perfil. A restauração exige ação explícita e nunca envia o prompt automaticamente. Um controle visível para esse recurso faz parte do próximo trabalho de configurações.

Leia [docs/SECURITY.md](./docs/SECURITY.md).

## Estrutura do repositório

    src/                Código do aplicativo Linux
    docs/               Arquitetura, segurança, build e compatibilidade
    scripts/            Utilitários de validação do repositório
    reference/swift/    Referências Swift somente leitura para a paridade restante
    .github/workflows/  CI Linux

O diretório reference/swift não é um produto macOS nem um alvo de build. Ele contém somente referências comportamentais ainda úteis para #17–#19.

## Documentação

- [Arquitetura](./docs/ARCHITECTURE.md)
- [Build](./docs/BUILDING.md)
- [Modelo de segurança](./docs/SECURITY.md)
- [Compatibilidade WebKitGTK](./docs/WEBKIT_COMPAT.md)
- [Matriz de portabilidade](./docs/PORTING_MATRIX.md)

## Estado do projeto

O Mado está na fase **P2 de paridade**. Empacotamento e distribuição ainda não foram finalizados; a direção futura é usar formatos padrão do Linux sem um runtime de navegador empacotado.

## Aviso

Mado é um projeto open source independente e não é afiliado, endossado ou patrocinado pela OpenAI.
