<template>
  <menu-layout>
    <template v-slot:header-left>
      <el-button :icon="Back" text bg size="small" @click="router.back()"/>
      开服协议
    </template>
    <template v-slot:header-right>
      <el-button type="primary" @click="confirmEula" :loading="isLoading" size="small">确认</el-button>
    </template>
    <template v-slot:content>
      <div class=page-section>
        <div class="container eulasection">
          <div class="row justify-content-center">
            <div class="col-md-10 col-lg-8">
              <div id=eulacontent data-doc-id=minecraft_eula data-language-code=zh-cn>
                <h1 id=minecraft->MINECRAFT 最终用户许可协议</h1>
                <p>为了保护 Minecraft（以下简称我们的"游戏"）以及我们社区的成员，我们需要这些最终用户许可条款为我们游戏的下载和使用制定一些规则。本许可是您和我们 (Mojang AB)
                  之间达成的法律协议，描述使用游戏的条款和条件。我们与您一样不喜欢阅读冗长的许可文档，因此我们尽可能将本协议的篇幅压缩到最短。如果您违反这些规则，我们可能会停止您使用我们的游戏。如果我们认为有必要，我们甚至会要求我们的律师提供帮助。
                </p>
                <p>如果您购买、下载、使用或玩我们的游戏，即表示您同意遵守这些最终用户许可协议（以下简称"EULA"）条款中的规定。如果您不愿或不能同意这些规定，则不得购买、下载、使用或玩我们的游戏。本 EULA 包含 <a
                    href=https://www.minecraft.net/en-us/terms#terms-mojang_general target="_blank">mojang.com
                    网站的使用条款（以下简称"帐户条款"）</a>、我们的<a
                    href=https://www.minecraft.net/terms#terms-brand_guidelines target="_blank">品牌和资产使用指南</a>以及<a
                    href="https://go.microsoft.com/fwlink/?LinkId=521839" target="_blank">我们的隐私政策</a>。通过接受本 EULA，即表示您也接受上述文档的所有条款，因此请仔细阅读本
                  EULA。</p>
                <h2 id=->一条重要规定</h2>
                <p>一条重要规定是除非我们明确同意，否则您不得分发我们创建的任何内容。"分发我们创建的任何内容"是指：</p>
                <ul>
                  <li>向任何其他人提供我们游戏的副本；</li>
                  <li>将我们创建的任何内容用于商业用途；</li>
                  <li>试图通过我们创建的任何内容赚钱；或者</li>
                  <li>允许其他人以不公平或不合理的方式访问我们创建的任何内容；</li>
                </ul>
                <p>除非我们明确同意您这么做。因此我们非常清楚地规定，"游戏"或"我们创建的内容"包括但不限于，我们游戏的客户端或服务器软件，还包括所有平台上的Minecraft和Minecraft: Java
                  Edition。它还包括游戏的更新、补丁、可下载内容、加载项或修改版本，这些内容的一部分；或我们提供的任何其他内容。</p>
                <p>除此之外，我们对您做的事情要求非常宽松，事实上我们鼓励您做一些很酷的事情，只是不要做我们规定不能做的那些事情即可。</p>
                <h2 id=->使用我们的游戏</h2>
                <p>您已被授予游戏的许可，因此您可以在自己的设备上使用它。</p>
                <p>下面我们还会赋予您做其他事情的有限权利，但我们必须划定分界线，以防人们越界。如果您想创建与我们已经创建的任何内容相关的内容，我们深感荣幸，但请确保它不会被解读为属于官方内容，并且符合本 EULA
                  及品牌和资产使用指南，最重要的是，不要将我们创建的任何内容用于商业用途。</p>
                <p>如果您违反本 EULA 的任一条款，我们可以撤销授权您使用我们的游戏的许可和权限。</p>
                <p>当您购买我们的游戏时，您会收到许可，该许可会授予您在自己的个人设备上安装游戏，并按照本 EULA
                  的规定在该设备上使用游戏的权限。该权限仅适用于您本人，因此不能将游戏（或其中的任一部分）分发给任何其他人。这还意味着您不能出售或出租游戏，或让其可供其他人访问，并且不能转让或转售任何许可密钥。但您可以赠送通过我们的官方礼品卡系统购买的礼品卡。您必须帮助我们防止盗版和欺诈并保护我们的游戏，这非常重要。还必须防止我们的社区成员购买我们游戏的盗版版本或欺诈性许可密钥，我们可以取消欺诈性密钥。
                </p>
                <p>如果您购买了我们的游戏，您可以尽情试用或通过改动、添加工具或插件（我们统称为"Mod"）来修改它。所谓"Mod"是指您或其他人的原创作品，不包含我们有版权的代码或内容的实质部分。当您将您的 Mod 与
                  Minecraft 软件进行组合时，我们将这种组合称为游戏的"Mod 化版本"。我们对哪些内容构成 Mod、哪些不构成 Mod 拥有最终决定权。您不得分发我们的游戏或软件的任何 Mod 化版本，如果您没有使用
                  Mod 来激怒或骚扰其他玩家，我们将不甚感激。基本上，Mod 可以分发；游戏客户端或服务器软件的 Mod 化版本的破解版本不可以分发。</p>
                <p>只要有正当理由，您可以使用游戏的屏幕截图和视频做任何您想做的事情。"只要有正当理由"是指您不能将其用于任何商业用途，或者做任何不公平或对我们的权利产生不利影响的事情，除非我们在本
                  EULA、品牌和资中使用指南或与您签订的特定协议中明确允许您这么做。不过，如果您向视频共享和流式播放网站上传游戏视频，您可以在其中放置广告。另外，不要只是剥离插图资源并四处传递，那样很无趣。</p>
                <p>总而言之，简单规定是除非我们明确同意，否则不要将我们创建的任何内容用于商业用途。当然，如果法律明确允许，例如根据"合理使用"或"公平交易"原则，那么您也可以这么做，但仅限适用法律允许的范围。</p>
                <p>为了确保游戏的完整性，我们需要所有游戏下载和更新都来自一个特许源：那就是我们。我们还需要确保第三方工具/服务看起来不是"官方的"，因为我们无法保证它们的质量。这是我们对 Minecraft
                  客户承担的责任的一部分。另外，还请务必阅读我们的品牌和资产使用指南。</p>
                <h2 id=->我们的游戏和其他内容的所有权</h2>
                <p>尽管我们授予您在设备上安装和畅玩我们游戏的权限，但我们仍然是游戏的所有者。我们还是我们的品牌以及游戏中包含的任何内容的所有者。因此，当您购买我们的游戏时，实际上您购买的是根据本 EULA
                  玩/使用我们的游戏的许可，而不是游戏本身。您拥有的与游戏及游戏安装有关的仅有权限是本 EULA 中规定的权限。</p>
                <p>您从头开始为游戏创建的任何 Mod 均归您所有（包括事先运行的 Mod 和内存中 Mod），您可以对它们执行任何所需操作，只要您不通过出售它们来赚钱/尝试通过它们赚钱，并且不分发游戏的 Mod
                  版本即可。记住，Mod 是您的原创作品，不包含我们代码或内容的实质部分。您只拥有您创建的内容，而不拥有我们的代码或内容。</p>
                <h2 id=->内容</h2>
                <p>
                  如果您在我们的游戏中或通过我们的游戏发布任何内容，则表示您同意授予我们使用、复制、修改、调整、分发和公开显示该内容的权限。该权限是不可撤销的，并且您还同意我们允许其他人使用、复制、修改、调整、分发和公开显示您的内容。您并不是放弃对您的内容的所有权，您只是授予我们和其他用户使用该内容的权限。例如，我们可能需要对您发布在我们网站上的内容进行复制、重新设置格式和分发，以便其他人可以阅读。如果您不想授予我们这些权限，请不要在我们的游戏中或通过我们的游戏发布任何内容。在发布任何内容之前请仔细考虑，因为内容将会被公开，甚至可能会被其他人以您不喜欢的方式使用。
                </p>
                <p>
                  如果您打算在我们的游戏中或通过我们的游戏发布内容，该内容不得冒犯他人或非法，必须真实，并且必须是您自己的作品。您不得使用我们的游戏发布的内容类型的一些示例包括：包含种族主义或恐同言论的帖子；欺凌或引诱性质的帖子；具有冒犯性或损坏我们或他人声誉的帖子；包含色情描写或其他人的作品或图片的帖子；或者假扮版主或试图欺骗或利用他人的帖子。
                </p>
                <p>您在我们的游戏中发布的任何内容还必须是您自己的作品，或者您必须具有这么做的权限或权利。您不得，并且您同意不会使用我们的游戏发布任何侵犯他人权利的内容。'</p>
                <p>我们保留根据我们自己的判断撤下任何内容的权利。</p>
                <p>
                  <strong>如果您在我们的游戏中与人们交谈，请当心。无论是您还是我们都很难确定人们说的哪些话是真的，甚至很难确定人们实际上是否与其所声称的身份相符。在披露有关您自己的信息之前，始终应该三思。</strong>
                </p>
                <h2 id=->更新</h2>
                <p>
                  我们可以不时发布升级、更新或补丁（我们将其统称为"更新"），但我们不必非得这么做。我们也没有义务对任何游戏提供持续支持或维护。当然，我们希望继续为我们的游戏发布新的更新，我们只是不能保证一定会这么做。更新中附带的更改可能不太适用于其他软件，例如
                  Mod。我们对此表示遗憾，但我们不会对此承担责任。如果您遇到这种情况，请尝试运行较旧版本。</p>
                <h2 id=->责任和管辖法律</h2>
                <p>本 EULA
                  中的条款不影响根据法律您可能享有的有关游戏的任何合法（法定）权利。您可能享有某些适用于您的法律规定不能免除的权利。这些条款中的任何规定都不会影响那些法定权利，即使这些规定可能与您的法定权利相冲突也是如此。当我们说"根据适用的法律"时，就是这个意思。
              </p>
              <p>
                根据适用的法律，当您获得我们游戏的副本时，我们按"现状"提供。更新也按"现状"提供。这意味着关于我们游戏的标准或质量，我们不向您做任何承诺，也不承诺我们的游戏将不会中断或不含错误。我们不对游戏可能造成的任何损失或损坏负责。您对游戏的质量和性能承担全部责任。您必须接受，我们可能会在游戏完成之前发布游戏，因此它们可能（而且经常会）包含
                BUG，但我们更愿意提前发布这些功能而不是让您等待它尽善尽美。如果您想通知我们某个潜在 BUG，我们在此处提供了专门的网站。</p>
              <p><strong>本 EULA
                  的解释和违反本协议的索赔受美国华盛顿州法律管辖，不受冲突法原则限制。所有其他索赔和争议（包括与消费者保护法、不正当竞争法以及侵权行为有关的索赔，以及与我们的游戏或网站相关的争议），应受您所在国家/地区的法律管辖，不受冲突法原则限制。</strong>
              </p>
              <h2 id=->终止</h2>
              <p>如果您违反任一条款，我们有权终止本 EULA。您也可以随时终止它；为此，您只需要从设备中卸载游戏即可，EULA 将自行终止。如果 EULA
                终止，您将不再享有本许可中赋予的任何游戏权利。不过，对于您在玩游戏的过程中自己创建的内容，您仍享有权利。即使在 EULA 终止后，有关"我们游戏的所有权"、"我们的责任"和"一般条款"的段落也将继续适用。
              </p>
              <h2 id=->一般条款</h2>
              <p>您当地的法律可能会赋予您本 EULA 无法更改的权利；如果是这样情况，本 EULA 将在法律允许的范围内适用。 本许可中的任何条款都不会限制我们对因我们的过失或欺诈性虚假陈述而导致的死亡或人身伤害的责任。
              </p>
              <p>如果有适当理由，我们可能会不时更改本 EULA，例如更改我们的游戏、惯例或法律义务。但这些更改只在法律允许的范围内生效。例如，如果您只在单玩家模式中玩游戏，并且不使用我们提供的更新，那么将适用旧
                EULA；但是如果您使用更新或者所用游戏的某些部分依赖我们提供的持续在线服务，那么新 EULA
                将适用。在后一种情况下，我们会在更改生效前通知您，方法是在我们的网站上张贴通知，或者通过其他合理的方式。虽然在这方面我们会尽量做到公平，但有时法律会有变化，或者有人会做一些影响游戏的其他用户的事情，因此我们需要禁止这种行为。
              </p>
              <p>
                如果您向我们提出有关我们的任一网站或游戏的建议，则该建议是免费的，我们没有义务接受或考虑。这意味着我们可以通过所需的任何方式使用或不使用您的建议，且不必向您支付报酬。如果您认为我们应为您的建议支付报酬，那么除非您提前告诉我们您希望得到报酬并且我们已经以书面形式回复，要求您提交该建议，否则请不要提交您的建议。
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </template>
</menu-layout>
</template>

<script setup lang="ts">
import { useRouter, useRoute, LocationQueryValue } from 'vue-router'
import MenuLayout from '../../layout/MenuLayout.vue'
import { Back } from '@element-plus/icons-vue'
import { setEula } from '../../api/server'
import { ref } from 'vue'
const router = useRouter()
const route = useRoute()
const instanceId = Number(route.query.instanceId as LocationQueryValue)

const isLoading = ref(false)
const confirmEula = () => {
  isLoading.value = true
  setEula({isAgree: true, instanceId: instanceId})
  .then(res => {
    router.back()
  }).finally(() => {
    isLoading.value = false
  })
}

</script>

<style scoped lang="scss">
h1 {
  font-size: 36px;
  font-weight: 700;
  line-height: 40px;
  margin-bottom: 12px;
}

h2 {
  font-size: 28px;
  font-weight: 700;
  line-height: 34px;
  margin-bottom: 12px;
}

ul {
  font-size: 16px;
  line-height: 24px;
  margin-bottom: 24px;
  margin-left: 15px;
  list-style: disc;
}

p {
  font-size: 16px;
  margin-bottom: 20px;
  line-height: 24px;
}

a {
  font-size: 16px;
  line-height: 24px;
  color: green;
  &:hover {
    color: white;
    background-color: black;
  }
}
</style>